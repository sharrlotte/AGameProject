use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::{
    block::{component::BlockData, resource::BlockType},
    ship::component::ShipData,
};

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct SchematicBlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

impl SchematicBlockData {
    pub fn to_block_data(&self) -> BlockData {
        BlockData {
            id: self.id,
            position: self.position,
            block_type: self.block_type,
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
            blocks: self
                .blocks
                .clone()
                .into_iter()
                .map(|b| b.to_block_data())
                .collect(),
        }
        .new()
    }
}
#[derive(Component, Default)]
pub struct Save {}
