use bevy::prelude::*;

use crate::constant::GRID_SIZE;
use crate::content::block::Rot90;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlockPlacement {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: String,
    pub rotation: Rot90,
}

impl BlockPlacement {
    pub fn new(position: IVec2, block_type: impl Into<String>, rotation: Rot90) -> Self {
        Self {
            id: Uuid::now_v7(),
            position,
            block_type: block_type.into(),
            rotation,
        }
    }

    pub fn position_pixel(&self) -> Vec2 {
        self.position.as_vec2() * GRID_SIZE
    }
}

#[derive(Clone, Debug)]
pub struct GridBlock {
    pub placement: BlockPlacement,
    pub entity: Entity,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRef {
    pub id: Uuid,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockOwner {
    pub body: Entity,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Disabled;
