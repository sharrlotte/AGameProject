use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    content::block::base::BlockType,
    world::base::{BlockData, ShipData},
};

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct SchematicBlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
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

#[derive(Message)]
pub struct SaveWorldEvent {
    pub path: PathBuf,
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub path: PathBuf,
}

#[derive(Component, Default)]
pub struct Save {
}
