use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::{
    block::{
        component::{Block, BlockData, BlockRegistry, Health, SingleShootPatternState, BurstShootPatternState},
        feature::turret::TurretState,
    },
    ship::{ShipData, ShipState},
    team::Team,
};
use crate::save::schema::{
    read_version, BlockSaveData, ShipSave, TurretSaveState, WorldSave, WORLD_SCHEMA_VERSION,
};
use crate::simulation::SimulationSet;
use crate::symbol::{Symbol, Symbols};

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct SchematicBlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: Symbol,
    pub rotation: u8,
}

impl SchematicBlockData {
    pub fn to_block_data(&self) -> BlockData {
        BlockData {
            id: self.id,
            position: self.position,
            block_type: self.block_type.clone(),
            rotation: self.rotation,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SchematicData {
    pub id: Uuid,
    pub filename: String,
    pub name: String,
    pub blocks: Vec<SchematicBlockData>,
}

impl SchematicData {
    pub fn to_ship_data(&self) -> ShipData {
        ShipData {
            id: self.id,
            blocks: self.blocks.clone().into_iter().map(|b| b.to_block_data()).collect(),
        }
        .new()
    }
}

#[derive(Component, Default)]
pub struct Save {}

#[derive(Message)]
pub struct SaveWorldEvent;

#[derive(Message)]
pub struct LoadWorldEvent;

pub fn save_dir() -> PathBuf {
    ProjectDirs::from("com", "somerandomdev", "agameproject")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            error!("Failed to resolve project directory, falling back to local ./agameproject");
            PathBuf::from("./agameproject")
        })
}

fn world_save_dir() -> PathBuf {
    save_dir().join("world")
}

pub fn load_saved_schematics(symbols: &mut Symbols) -> Vec<SchematicData> {
    let schematics_dir = save_dir().join("schematics");
    let mut schematics = Vec::new();

    if let Ok(entries) = fs::read_dir(schematics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.ends_with(".json")
            {
                let path = save_dir().join("schematics").join(name);
                if let Ok(json) = fs::read_to_string(path)
                    && let Ok(mut ship) = serde_json::from_str::<SchematicData>(&json)
                {
                    ship.filename = name.to_string().replace(".json", "");
                    schematics.push(ship);
                }
            }
        }
    }

    while schematics.len() < 5 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos().to_string())
            .unwrap_or_else(|_| "0".to_string());

        schematics.push(SchematicData {
            id: Uuid::now_v7(),
            filename: timestamp,
            name: format!("ship_{}", schematics.len()),
            blocks: Vec::new(),
        });
    }

    for ship in &mut schematics {
        if ship.blocks.len() <= 2 {
            ship.blocks.clear();
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: symbols.intern("core"),
                position: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: symbols.intern("laser_turret"),
                position: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    schematics
}

pub fn save_schematic(data: &SchematicData) -> Result<(), std::io::Error> {
    let schematics_dir = save_dir().join("schematics");
    let path = schematics_dir.join(format!("{}.json", data.filename));
    fs::create_dir_all(&schematics_dir)?;
    let json = serde_json::to_string_pretty(&data)?;
    fs::write(path, json)
}

/// Single entry point for spawning a ship from a versioned RON save.
pub fn spawn_ship_from_save(
    commands: &mut Commands,
    save: &ShipSave,
    registry: &BlockRegistry,
    config: &GameConfigValue,
) -> Option<Entity> {
    let blocks: Vec<BlockData> = save
        .blocks
        .iter()
        .map(|b| BlockData {
            id: Uuid::now_v7(),
            position: b.position,
            block_type: b.block_type.clone(),
            rotation: b.rotation,
        })
        .collect();

    let team = Team::new(save.team);
    let (ship, block_entities) = ShipData {
        id: save.id,
        blocks,
    }
    .spawn(commands, registry, team, Transform::default(), config)?;

    for (entity, saved) in block_entities.iter().zip(save.blocks.iter()) {
        commands.entity(*entity).insert(Health {
            health: saved.health,
            max_health: saved.max_health,
        });

        if let Some(turret) = &saved.turret_state {
            commands.entity(*entity).insert(TurretState {
                active: turret.active,
                target: None,
                shoot_at: turret.shoot_at,
            });
        }

        if let Some(state) = &saved.single_pattern {
            commands.entity(*entity).insert(state.clone());
        }
        
        if let Some(state) = &saved.burst_pattern {
            commands.entity(*entity).insert(state.clone());
        }
    }

    Some(ship)
}

pub fn handle_save_world_event(
    mut events: MessageReader<SaveWorldEvent>,
    ships: Query<(Entity, &ShipState, &Team)>,
    blocks: Query<(Entity, &Block, &Health)>,
    turrets: Query<&TurretState>,
    singles: Query<&SingleShootPatternState>,
    bursts: Query<&BurstShootPatternState>,
    parents: Query<&ChildOf>,
    battle_time: Res<crate::mission::BattleResult>,
) {
    for _ in events.read() {
        let dir = world_save_dir();
        let ships_dir = dir.join("ships");
        if let Err(e) = fs::create_dir_all(&ships_dir) {
            warn!("Failed to create world save dir: {e}");
            continue;
        }

        let mut ship_ids = Vec::new();
        let mut teams = Vec::new();

        for (ship_entity, _, team) in &ships {
            let id = ship_entity.to_bits();
            let ship_uuid = Uuid::from_u128(id as u128);
            ship_ids.push(ship_uuid);
            if !teams.contains(&team.id) {
                teams.push(team.id);
            }

            let mut block_saves = Vec::new();
            for (block_entity, block, health) in &blocks {
                let is_child_of_ship = parents
                    .get(block_entity)
                    .map(|p| p.parent() == ship_entity)
                    .unwrap_or(false);
                if !is_child_of_ship {
                    continue;
                }

                let turret_state = turrets.get(block_entity).ok().map(|s| TurretSaveState {
                    active: s.active,
                    shoot_at: s.shoot_at,
                });
                let single = singles.get(block_entity).ok().cloned();
                let burst = bursts.get(block_entity).ok().cloned();

                block_saves.push(BlockSaveData {
                    position: block.position,
                    block_type: block.block_type.clone(),
                    rotation: block.rotation,
                    health: health.health,
                    max_health: health.max_health,
                    turret_state,
                    single_pattern: single,
                    burst_pattern: burst,
                });
            }

            let ship_save = ShipSave {
                version: WORLD_SCHEMA_VERSION,
                id: ship_uuid,
                team: team.id,
                blocks: block_saves,
            };
            let path = ships_dir.join(format!("{ship_uuid}.ron"));
            match ron::ser::to_string_pretty(&ship_save, ron::ser::PrettyConfig::default()) {
                Ok(text) => {
                    if let Err(e) = fs::write(path, text) {
                        warn!("Failed to write ship save: {e}");
                    }
                }
                Err(e) => warn!("Failed to serialize ship save: {e}"),
            }
        }

        let world = WorldSave {
            version: WORLD_SCHEMA_VERSION,
            teams,
            elapsed_secs: battle_time.started_at.unwrap_or(0.0),
            ship_ids,
        };
        match ron::ser::to_string_pretty(&world, ron::ser::PrettyConfig::default()) {
            Ok(text) => {
                if let Err(e) = fs::write(dir.join("world.ron"), text) {
                    warn!("Failed to write world.ron: {e}");
                }
            }
            Err(e) => warn!("Failed to serialize world.ron: {e}"),
        }
    }
}

pub fn handle_load_world_event(
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    effects: Query<Entity, With<Save>>,
) {
    for _ in events.read() {
        let dir = world_save_dir();
        let world_text = match fs::read_to_string(dir.join("world.ron")) {
            Ok(text) => text,
            Err(e) => {
                warn!("Failed to read world save: {e}");
                continue;
            }
        };
        if let Err(e) = read_version(&world_text) {
            warn!("{e}");
            continue;
        }
        let world: WorldSave = match ron::from_str(&world_text) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to parse world.ron: {e}");
                continue;
            }
        };

        // Effect-like entities are not in the schema; remove all of them so a
        // loaded world cannot leak missiles/lasers/drops/damage text.
        for entity in &effects {
            commands.entity(entity).despawn();
        }

        for id in world.ship_ids {
            let path = dir.join("ships").join(format!("{id}.ron"));
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(e) => {
                    warn!("Failed to read ship save {id}: {e}");
                    continue;
                }
            };
            if let Err(e) = read_version(&text) {
                warn!("{e}");
                continue;
            }
            let ship: ShipSave = match ron::from_str(&text) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to parse ship save {id}: {e}");
                    continue;
                }
            };
            spawn_ship_from_save(&mut commands, &ship, &registry, &config);
        }
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .add_systems(
                Update,
                (
                    handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
                    handle_load_world_event.run_if(on_message::<LoadWorldEvent>),
                )
                    .in_set(SimulationSet::Apply),
            );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::schema::*;

    #[test]
    fn ship_save_roundtrip_and_version() {
        let save = ShipSave {
            version: WORLD_SCHEMA_VERSION,
            id: Uuid::now_v7(),
            team: 0,
            blocks: vec![BlockSaveData {
                position: IVec2::new(2, 3),
                block_type: Symbol::from("core"),
                rotation: 0,
                health: 4000.0,
                max_health: 5000.0,
                turret_state: Some(TurretSaveState { active: true, shoot_at: None }),
                single_pattern: None,
                burst_pattern: None,
            }],
        };
        let text = ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();
        assert_eq!(read_version(&text).unwrap(), WORLD_SCHEMA_VERSION);
        let back: ShipSave = ron::from_str(&text).unwrap();
        assert_eq!(back, save);

        let bad = text.replace("version: 1", "version: 99");
        assert!(read_version(&bad).is_err());
    }
}
