use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{
    content::{
        block::base::{BlockState, BlockType, Health, ShootPatternState},
        ship::base::ShipState,
        turret::base::TurretState,
    },
    save::base::{LoadWorldEvent, SaveWorldEvent, SchematicBlockData, SchematicData},
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
pub struct Save {
    ships: Vec<ShipSave>,
    blocks: Vec<BlockSave>,
    turrets: Vec<TurretSave>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShipSave {
    id: Uuid,
    translation: Vec3,
    rotation: Quat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockSave {
    id: Uuid,
    parent: Uuid,
    block_type: BlockType,
    translation: Vec3,
    rotation: Quat,
    health: Health,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TurretSave {
    id: Uuid,
    parent: Uuid,
    translation: Vec3,
    rotation: Quat,
    state: TurretState,
    shoot_pattern: ShootPatternState,
}

pub fn handle_save_world_event(
    ship_query: Query<(Entity, &ShipState, &Transform)>,
    block_query: Query<(Entity, &BlockState, &Transform, &Health)>,
    turret_query: Query<(Entity, &TurretState, &Transform, &ShootPatternState)>,
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

    for event in events.read() {
        let ships = ship_query
            .iter()
            .map(|(entity, state, transform)| ShipSave {
                id: get_id(&mut ids, entity),
                translation: transform.translation,
                rotation: transform.rotation,
            })
            .collect::<Vec<_>>();

        info!("saving {} ships", ships.len());

        let blocks = block_query
            .iter()
            .map(|(entity, state, transform, health)| BlockSave {
                id: get_id(&mut ids, entity),
                parent: get_parent(&ids, entity),
                translation: transform.translation,
                rotation: transform.rotation,
                block_type: state.block_type,
                health: health.clone(),
            })
            .collect::<Vec<_>>();

        info!("saving {} blocks", blocks.len());

        let turrets = turret_query
            .iter()
            .map(|(entity, state, transform, shoot_pattern)| TurretSave {
                id: get_id(&mut ids, entity),
                parent: get_parent(&ids, entity),
                translation: transform.translation,
                rotation: transform.rotation,
                state: state.clone(),
                shoot_pattern: shoot_pattern.clone(),
            })
            .collect::<Vec<_>>();

        info!("saving {} turrets", turrets.len());

        let path = event.path.clone();

        let save = Save {
            ships,
            blocks,
            turrets,
        };

        if let Err(e) = write_world_save(&path, save) {
            error!("Failed to write world save: {:?}", e);
        }
    }
}

fn write_world_save(path: &PathBuf, save: Save) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&save)?;

    fs::write(&path, json)?;
    return Ok(());
}

pub fn handle_load_world_event(mut commands: Commands, mut events: MessageReader<LoadWorldEvent>) {
    for event in events.read() {
        let path = event.path.clone();

        let data_string = match fs::read_to_string(&path) {
            Ok(data) => data,
            Err(error) => {
                error!("Failed to read world save: {:?} {:?}", path, error);
                continue;
            }
        };

        let save = match serde_json::from_str::<Save>(&data_string) {
            Ok(save) => save,
            Err(error) => {
                error!("Failed to parse world save: {:?} {:?}", path, error);
                continue;
            }
        };

        let Save {
            ships,
            blocks,
            turrets,
        } = save;

        info!("loading {} ships", ships.len());
        info!("loading {} blocks", blocks.len());
        info!("loading {} turrets", turrets.len());
    }
}
