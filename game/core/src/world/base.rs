use bevy::math::{IVec2, Vec2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::base::BlockType;

pub struct WorldContext {
    pub ships: Vec<ShipData>,
    pub blocks: Vec<BlockData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShipData {
    pub id: Uuid,
    pub position: Vec2,
    pub rotation: u8,
    pub blocks: Vec<BlockData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}
