use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Error, path::PathBuf};

use crate::{
    content::block::base::BlockType,
    world::base::{BlockData, ShipData},
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
                        if let Ok(ship) = serde_json::from_str::<SchematicData>(&json) {
                            schematics.push(ship);
                        }
                    }
                }
            }
        }
    }

    while schematics.len() < 3 {
        schematics.push(SchematicData {
            id: schematics.len(),
            name: format!("ship_{}", schematics.len()),
            blocks: Vec::new(),
        });
    }

    for ship in &mut schematics {
        if ship.blocks.len() <= 2 {
            ship.blocks.clear();
            ship.blocks.push(SchematicBlockData {
                id: 0,
                block_type: BlockType::Core,
                position: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(SchematicBlockData {
                id: 1,
                block_type: BlockType::LazerTurret,
                position: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    return schematics;
}

pub fn save_schematic(data: &SchematicData) -> Result<(), Error> {
    let schematics_dir = save_dir().join("schematics");
    let filename = format!("{}.json", data.name);
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

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct SchematicBlockData {
    pub id: usize,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SchematicData {
    pub id: usize,
    pub name: String,
    pub blocks: Vec<SchematicBlockData>,
}

impl SchematicData {
    pub fn to_ship_data(&self) -> ShipData {
        ShipData {
            id: self.id,
            position: Vec2::new(0.0, 0.0),
            rotation: 0,
            blocks: self
                .blocks
                .clone()
                .into_iter()
                .map(|b| BlockData {
                    id: b.id,
                    position: b.position,
                    block_type: b.block_type,
                    rotation: b.rotation,
                })
                .collect(),
        }
    }
}

pub fn save_map() {}
