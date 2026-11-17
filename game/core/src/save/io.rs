use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::{
    block::{
        BlockPlacement, BlockRef, BlockRegistry, BurstShootPatternState, Health,
        SingleShootPatternState,
        feature::turret::TurretState,
    },
    ship::{ShipDesign, ShipGrid},
    team::Team,
};
use crate::mission::{BattlePhase, BattleResult};
use crate::resource::{CameraTarget, ControlledShip, PendingLoad, SceneState};
use crate::save::document::{read_document, write_document};
use crate::save::schema::{
    BlockSaveData, ShipSave, TurretSaveState, WorldSave, WORLD_SCHEMA_VERSION,
};
use crate::simulation::SimulationSet;

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct SchematicBlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: String,
    pub rotation: u8,
}

impl SchematicBlockData {
    pub fn to_placement(&self) -> BlockPlacement {
        BlockPlacement {
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
    pub fn to_design(&self) -> ShipDesign {
        ShipDesign {
            version: 1,
            id: self.id,
            name: self.name.clone(),
            blocks: self
                .blocks
                .clone()
                .into_iter()
                .map(|b| b.to_placement())
                .collect(),
        }
    }
}

#[derive(Message)]
pub struct SaveWorldEvent;

/// Request a world load. Handled as a state transition so the outgoing scene
/// is despawned by `DespawnOnExit` before the snapshot is spawned.
#[derive(Message)]
pub struct LoadWorldEvent {
    pub slot: u8,
}

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

pub fn load_saved_schematics() -> Vec<SchematicData> {
    let schematics_dir = save_dir().join("schematics");
    let mut schematics = Vec::new();

    if let Ok(entries) = fs::read_dir(&schematics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.ends_with(".ron")
            {
                let path = schematics_dir.join(name);
                if let Ok(text) = fs::read_to_string(path)
                    && let Ok(mut ship) = ron::from_str::<SchematicData>(&text)
                {
                    ship.filename = name.trim_end_matches(".ron").to_string();
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
                block_type: "core".to_string(),
                position: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: "laser_turret".to_string(),
                position: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    schematics
}

pub fn save_schematic(data: &SchematicData) -> Result<(), std::io::Error> {
    let schematics_dir = save_dir().join("schematics");
    let path = schematics_dir.join(format!("{}.ron", data.filename));
    fs::create_dir_all(&schematics_dir)?;
    let text = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    fs::write(path, text)
}

/// Single entry point for spawning a ship from a versioned RON save.
pub fn spawn_ship_from_save(
    commands: &mut Commands,
    save: &ShipSave,
    registry: &BlockRegistry,
    config: &GameConfigValue,
) -> Option<Entity> {
    let design = ShipDesign {
        version: 1,
        id: save.id,
        name: String::new(),
        blocks: save
            .blocks
            .iter()
            .map(|b| BlockPlacement {
                id: Uuid::now_v7(),
                position: b.position,
                block_type: b.block_type.clone(),
                rotation: b.rotation,
            })
            .collect(),
    };

    let team = Team::new(save.team);
    let (ship, block_entities) =
        design.spawn(commands, registry, team, Transform::default(), config)?;

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
    ships: Query<(Entity, &ShipGrid, &Team)>,
    blocks: Query<(&BlockRef, &Health)>,
    turrets: Query<&TurretState>,
    singles: Query<&SingleShootPatternState>,
    bursts: Query<&BurstShootPatternState>,
    battle_time: Res<crate::mission::BattleResult>,
) {
    for _ in events.read() {
        let dir = world_save_dir();
        let ships_dir = dir.join("ships");
        let mut ship_ids = Vec::new();
        let mut teams = Vec::new();

        for (ship_entity, grid, team) in &ships {
            let id = ship_entity.to_bits();
            let ship_uuid = Uuid::from_u128(id as u128);
            ship_ids.push(ship_uuid);
            if !teams.contains(&team.id) {
                teams.push(team.id);
            }

            let mut block_saves = Vec::new();
            for grid_block in grid.blocks() {
                let block_entity = grid_block.entity;
                let Ok((_, health)) = blocks.get(block_entity) else {
                    continue;
                };
                let turret_state = turrets.get(block_entity).ok().map(|s| TurretSaveState {
                    active: s.active,
                    shoot_at: s.shoot_at,
                });
                let single = singles.get(block_entity).ok().cloned();
                let burst = bursts.get(block_entity).ok().cloned();

                block_saves.push(BlockSaveData {
                    position: grid_block.placement.position,
                    block_type: grid_block.placement.block_type.clone(),
                    rotation: grid_block.placement.rotation,
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
            if let Err(e) = write_document(&ships_dir.join(format!("{ship_uuid}.ron")), &ship_save)
            {
                warn!("Failed to write ship save: {e}");
            }
        }

        let world = WorldSave {
            version: WORLD_SCHEMA_VERSION,
            teams,
            elapsed_secs: battle_time.started_at.unwrap_or(0.0),
            ship_ids,
        };
        if let Err(e) = write_document(&dir.join("world.ron"), &world) {
            warn!("Failed to write world.ron: {e}");
        }
    }
}

/// Convert a load request into a scene transition. The snapshot is spawned on
/// the next `SceneState::Game` entry, after the outgoing scene has been torn
/// down by `DespawnOnExit(SceneState::Game)`.
pub fn handle_load_request(
    mut events: MessageReader<LoadWorldEvent>,
    mut pending: ResMut<PendingLoad>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    for event in events.read() {
        pending.request(event.slot);
        next_scene.set(SceneState::Loading);
    }
}

/// Move from `Loading` into `Game` once a load has been requested.
pub fn begin_loaded_game(
    pending: Res<PendingLoad>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    if pending.is_pending() {
        next_scene.set(SceneState::Game);
    }
}

/// Spawn the requested snapshot when `Game` is entered. Runs alongside
/// `setup_battle`, which skips itself while a load is pending.
pub fn spawn_loaded_world(
    mut commands: Commands,
    pending: Res<PendingLoad>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    time: Res<Time>,
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
) {
    if !pending.is_pending() {
        return;
    }

    // Deferred so other OnEnter(Game) systems still observe the pending flag
    // this frame and skip mission setup.
    commands.insert_resource(PendingLoad::default());
    commands.insert_resource(CameraTarget::default());
    commands.insert_resource(ControlledShip::default());

    let dir = world_save_dir();
    let world: WorldSave = match read_document(&dir.join("world.ron")) {
        Ok(world) => world,
        Err(e) => {
            warn!("Failed to read world save: {e}");
            return;
        }
    };

    for id in world.ship_ids {
        let ship: ShipSave = match read_document(&dir.join("ships").join(format!("{id}.ron"))) {
            Ok(ship) => ship,
            Err(e) => {
                warn!("Failed to read ship save {id}: {e}");
                continue;
            }
        };
        spawn_ship_from_save(&mut commands, &ship, &registry, &config);
    }

    *result = BattleResult {
        started_at: Some(time.elapsed_secs()),
        outcome: None,
        duration_secs: 0.0,
    };
    phase.set(BattlePhase::Active);
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .init_resource::<PendingLoad>()
            .add_systems(
                Update,
                (
                    handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
                    handle_load_request.run_if(on_message::<LoadWorldEvent>),
                )
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                begin_loaded_game
                    .run_if(in_state(SceneState::Loading))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(OnEnter(SceneState::Game), spawn_loaded_world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::document::parse_document;
    use crate::save::schema::*;

    #[test]
    fn ship_save_roundtrip_and_version() {
        let save = ShipSave {
            version: WORLD_SCHEMA_VERSION,
            id: Uuid::now_v7(),
            team: 0,
            blocks: vec![BlockSaveData {
                position: IVec2::new(2, 3),
                block_type: "core".to_string(),
                rotation: 0,
                health: 4000.0,
                max_health: 5000.0,
                turret_state: Some(TurretSaveState {
                    active: true,
                    shoot_at: None,
                }),
                single_pattern: None,
                burst_pattern: None,
            }],
        };
        let text = ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();
        let back: ShipSave = parse_document(&text).unwrap();
        assert_eq!(back, save);

        let bad = text.replace("version: 1", "version: 99");
        assert!(parse_document::<ShipSave>(&bad).is_err());
    }

    #[test]
    fn schematic_ron_roundtrip() {
        let schematic = SchematicData {
            id: Uuid::now_v7(),
            filename: "test".to_string(),
            name: "Test".to_string(),
            blocks: vec![SchematicBlockData {
                id: Uuid::now_v7(),
                position: IVec2::new(1, 2),
                block_type: "core".to_string(),
                rotation: 0,
            }],
        };
        let text = ron::ser::to_string(&schematic).unwrap();
        let back: SchematicData = ron::from_str(&text).unwrap();
        assert_eq!(back.blocks[0].block_type, "core");
    }
}
