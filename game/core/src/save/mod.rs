use bevy::prelude::*;
use directories::ProjectDirs;
use std::{fs, io::Error, path::PathBuf};

use crate::content::structures::{
    ship::{BlockData, ShipData},
    types::BlockType,
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

pub fn load_saved_ships() -> Vec<ShipData> {
    let ships_dir = save_dir().join("ships");

    let mut ships = Vec::new();

    if let Ok(entries) = fs::read_dir(ships_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let path = save_dir().join("ships").join(name);
                    if let Ok(json) = fs::read_to_string(path) {
                        if let Ok(ship) = serde_json::from_str::<ShipData>(&json) {
                            ships.push(ship);
                        }
                    }
                }
            }
        }
    }

    while ships.len() < 3 {
        ships.push(ShipData {
            name: format!("ship_{}", ships.len()),
            blocks: Vec::new(),
        });
    }

    for ship in &mut ships {
        if ship.blocks.len() <= 2 {
            ship.blocks.clear();
            ship.blocks.push(BlockData {
                id: 0,
                block_type: BlockType::Core,
                pos: IVec2::new(0, 0),
                rotation: 0,
            });
            ship.blocks.push(BlockData {
                id: 1,
                block_type: BlockType::LazerTurret,
                pos: IVec2::new(1, 3),
                rotation: 0,
            });
        }
    }

    return ships;
}

pub fn save_ship(data: &ShipData) -> Result<(), Error> {
    let ships_dir = save_dir().join("ships");
    let filename = format!("{}.json", data.name);
    let path = ships_dir.join(&filename);

    // Create ships directory if it doesn't exist
    if !ships_dir.exists() {
        if let Err(e) = fs::create_dir_all(&ships_dir) {
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

pub fn _delete_ship(name: &str) {
    let ships_dir = save_dir().join("ships");
    let filename = format!("{}.json", name);
    let path = ships_dir.join(&filename);

    if path.exists() {
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete ship: {}", e);
        }
    }
}
