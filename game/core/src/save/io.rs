use bevy::{asset::ron, platform::collections::HashSet, prelude::*};
use directories::ProjectDirs;
use std::{
    any::TypeId,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use crate::{
    content::{
        block::base::{BlockType, ShootPatternState},
        ship::base::ShipState,
        turret::base::TurretState,
    },
    save::base::{SchematicBlockData, SchematicData},
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

pub type SaveResult = Result<Saved, SaveError>;

#[derive(Component, Default, Debug, Clone)]
pub struct Save;

#[derive(Event)]
pub struct Saved {
    pub scene: DynamicScene,
}

impl Saved {
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.scene.entities.iter().map(|de| de.entity)
    }
}

#[derive(Debug)]
pub enum SaveError {
    Ron(ron::Error),
    Io(std::io::Error),
}

impl From<ron::Error> for SaveError {
    fn from(e: ron::Error) -> Self {
        Self::Ron(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn save_world(world: &mut World, name: &String) -> SaveResult {
    let path = save_dir().join("saves").join(name);

    let entities: Vec<_> = world
        .query_filtered::<Entity, With<Save>>()
        .iter(world)
        .collect();

    info!("saving {} entities", entities.len());

    let scene = DynamicSceneBuilder::from_world(world)
        .with_component_filter(SceneFilter::Allowlist(HashSet::from([
            TypeId::of::<ShipState>(),
            TypeId::of::<TurretState>(),
            TypeId::of::<ShootPatternState>(),
        ])))
        .extract_entities(entities.iter().copied())
        .build();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let type_registry = world.resource::<AppTypeRegistry>().read();
    let data = scene.serialize(&type_registry)?;
    std::fs::write(&path, data.as_bytes())?;
    debug!("saved into file: {path:?}");

    return Ok(Saved { scene });
}
