use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Debug,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{
    content::{
        block::{
            base::{Block, BlockState, BlockType, Health, SingleShootPatternState},
            registry::BlockRegistry,
        },
        ship::base::ShipState,
        team::Team,
        turret::base::TurretState,
    },
    save::base::{LoadWorldEvent, SaveWorldEvent, SchematicBlockData, SchematicData},
    world::base::{BlockData, ShipData, render_block_base},
};


pub fn save_dir() -> PathBuf {
    let project_dirs = ProjectDirs::from(
        "com",           // qualifier
        "somerandomdev", // organization
        "agameproject",  // application
    )
    .expect("Failed to get project directory");

    project_dirs.data_dir().to_path_buf()
}

pub fn load_saved_schematics() -> Vec<SchematicData> {
    let schematics_dir = save_dir().join("schematics");

    let mut schematics = Vec::new();

    if let Ok(entries) = fs::read_dir(schematics_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let path = save_dir().join("schematics").join(name);
                    if let Ok(json) = fs::read_to_string(path) {
                        if let Ok(mut ship) = serde_json::from_str::<SchematicData>(&json) {
                            ship.filename = name.to_string().replace(".json", "");
                            schematics.push(ship);
                        }
                    }
                }
            }
        }
    }

    while schematics.len() < 3 {
        let timetamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        schematics.push(SchematicData {
            id: Uuid::now_v7(),
            filename: timetamp,
            name: format!("ship_{}", schematics.len()),
            blocks: Vec::new(),
        });
    }

    for ship in &mut schematics {
        if ship.blocks.len() <= 2 {
            ship.blocks.clear();
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: BlockType::Core,
                position: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(SchematicBlockData {
                id: Uuid::now_v7(),
                block_type: BlockType::LazerTurret,
                position: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    return schematics;
}

pub fn save_schematic(data: &SchematicData) -> Result<(), std::io::Error> {
    let schematics_dir = save_dir().join("schematics");
    let filename = format!("{}.json", data.filename);
    let path = schematics_dir.join(&filename);

    // Create schematics directory if it doesn't exist
    if !schematics_dir.exists() {
        if let Err(e) = fs::create_dir_all(&schematics_dir) {
            return Err(e);
        }
    }

    match serde_json::to_string_pretty(&data) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                return Err(e);
            } else {
                info!("Ship saved at: {}", path.display());
                return Ok(());
            }
        }
        Err(e) => return Err(e.into()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipSave {
    id: Uuid,
    state: ShipState,
    team: Team,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockSave {
    id: Uuid,
    parent: Uuid,
    block_type: BlockType,
    position: IVec2,
    rotation: u8,
    state: BlockState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TurretSave {
    id: Uuid,
    parent: Uuid,
    state: TurretState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentWrapper<T: Component + Clone> {
    parent: Uuid,
    component: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    ships: Vec<ShipSave>,
    blocks: Vec<BlockSave>,
    turrets: Vec<TurretSave>,
    transforms: Vec<ParentWrapper<Transform>>,
    healths: Vec<ParentWrapper<Health>>,
    shoot_patterns: ShootPatternSave,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShootPatternSave {
    single: Vec<ParentWrapper<SingleShootPatternState>>,
}

pub fn handle_save_world_event(
    transform_query: Query<(Entity, &Transform)>,
    ship_query: Query<(Entity, &ShipState, &Team)>,
    block_query: Query<(Entity, &Block, &BlockState)>,
    turret_query: Query<(Entity, &TurretState)>,
    health_query: Query<(Entity, &Health)>,
    singel_shoot_pattern_query: Query<(Entity, &SingleShootPatternState)>,
    parents: Query<(Entity, &ChildOf)>,
    mut events: MessageReader<SaveWorldEvent>,
) {
    let mut ids: HashMap<Entity, Uuid> = HashMap::new();

    let get_parent = |ids: &HashMap<Entity, Uuid>, entity: Entity| -> Uuid {
        let parent_id = &parents
            .get(entity)
            .expect(format!("Entity has no parent: {:?}", entity).as_str())
            .1
            .parent();

        return *ids.get(parent_id).expect(
            format!(
                "Parent is not in child: {}, parent: {}, ids: {:?}",
                entity, parent_id, ids
            )
            .as_str(),
        );
    };

    let get_id = |ids: &mut HashMap<Entity, Uuid>, entity: Entity| -> Uuid {
        if ids.contains_key(&entity) {
            panic!("Entity {} already has an id", entity);
        }

        let id = Uuid::now_v7();
        ids.insert(entity, id);
        return id;
    };

    fn clone_query<T: Component + Debug + Clone>(
        ids: &HashMap<Entity, Uuid>,
        query: &Query<(Entity, &T)>,
    ) -> Vec<ParentWrapper<T>> {
        let mut result: Vec<ParentWrapper<T>> = Vec::new();

        for (entity, component) in query.iter() {
            let Some(&parent) = ids.get(&entity) else {
                warn!("Entity {} has no parent", entity);
                continue;
            };

            result.push(ParentWrapper::<T> {
                parent,
                component: component.clone(),
            });
        }

        return result;
    }

    for event in events.read() {
        let ships = ship_query
            .iter()
            .map(|(entity, state, team)| ShipSave {
                id: get_id(&mut ids, entity),
                state: state.clone(),
                team: team.clone(),
            })
            .collect::<Vec<_>>();

        info!("saving {} ships", ships.len());

        let blocks = block_query
            .iter()
            .map(|(entity, block, state)| BlockSave {
                id: get_id(&mut ids, entity),
                parent: get_parent(&ids, entity),
                block_type: block.block_type,
                position: block.position,
                rotation: block.rotation,
                state: state.clone(),
            })
            .collect::<Vec<_>>();

        info!("saving {} blocks", blocks.len());

        let turrets = turret_query
            .iter()
            .map(|(entity, state)| TurretSave {
                id: get_id(&mut ids, entity),
                parent: get_parent(&ids, entity),
                state: state.clone(),
            })
            .collect::<Vec<_>>();

        info!("saving {} turrets", turrets.len());

        let path = event.path.clone();

        let save = SaveData {
            ships,
            blocks,
            turrets,
            transforms: clone_query(&ids, &transform_query),
            healths: clone_query(&ids, &health_query),
            shoot_patterns: ShootPatternSave {
                single: clone_query(&ids, &singel_shoot_pattern_query),
            },
        };

        if let Err(e) = write_world_save(&path, save) {
            warn!("Failed to write world save: {:?}", e);
        }
    }
}

fn write_world_save(path: &PathBuf, save: SaveData) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&save)?;

    fs::write(&path, json)?;
    return Ok(());
}

fn spawn_components<T: Component + Clone>(
    commands: &mut Commands,
    ids: &HashMap<Uuid, Entity>,
    components: Vec<ParentWrapper<T>>,
) {
    for data in components {
        let Some(entity) = ids.get(&data.parent) else {
            warn!("Failed to get entity: {:?}", data.parent);
            continue;
        };

        commands.entity(*entity).insert(data.component);
    }
}

pub fn handle_load_world_event(
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    for event in events.read() {
        let path = event.path.clone();
        let mut ids: HashMap<Uuid, Entity> = HashMap::new();

        let data_string = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(warn) => {
                warn!("Failed to read world save: {:?} {:?}", path, warn);
                continue;
            }
        };

        let save = match serde_json::from_str::<SaveData>(&data_string) {
            Ok(save) => save,
            Err(warn) => {
                warn!("Failed to parse world save: {:?} {:?}", path, warn);
                continue;
            }
        };

        let SaveData {
            ships,
            blocks,
            turrets,
            healths,
            shoot_patterns,
            transforms,
        } = save;

        info!("loading {} ships", ships.len());
        info!("loading {} blocks", blocks.len());
        info!("loading {} turrets", turrets.len());

        for ship_data in ships {
            let blocks = blocks
                .iter()
                .filter(|b| b.parent == ship_data.id)
                .map(|b| BlockData {
                    id: b.id,
                    position: b.position,
                    block_type: b.block_type,
                    rotation: b.rotation,
                })
                .collect::<Vec<_>>();

            let data = ShipData {
                id: ship_data.id,
                blocks,
            };

            let ship = data.spawn(
                &mut commands,
                &registry,
                &assets,
                ship_data.team,
                Transform::default(),
            );

            ids.insert(ship_data.id, ship);

            for block_data in &data.blocks {
                let Some(def) = registry.get(&block_data.block_type) else {
                    warn!("Failed to get block def: {:?}", block_data.block_type);
                    continue;
                };

                let block = render_block_base(
                    &mut commands,
                    ship,
                    def,
                    block_data.id,
                    block_data.position,
                    ship_data.team,
                );

                ids.insert(block_data.id, block);
            }
        }

        for turret_data in turrets {
            let Some(parent) = ids.get(&turret_data.parent) else {
                warn!("Failed to get parent block: {:?}", turret_data.parent);
                continue;
            };

            commands.entity(*parent).insert(TurretState {
                potential_targets: HashSet::new(),
                active: turret_data.state.active,
                target: turret_data.state.target,
                shoot_at: turret_data.state.shoot_at,
            });
        }

        spawn_components(&mut commands, &ids, transforms);
        spawn_components(&mut commands, &ids, healths);
        spawn_components(&mut commands, &ids, shoot_patterns.single);
    }
}
