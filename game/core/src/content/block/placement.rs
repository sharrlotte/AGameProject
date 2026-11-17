use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Static description of one block in a ship layout.
///
/// `ShipGrid` is authoritative for placement and identity; this type is the
/// only block description that travels through spawn, editor, save, and
/// mutation paths.
#[derive(Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlockPlacement {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: String,
    pub rotation: u8,
}

impl BlockPlacement {
    pub fn new(position: IVec2, block_type: impl Into<String>, rotation: u8) -> Self {
        Self {
            id: Uuid::now_v7(),
            position,
            block_type: block_type.into(),
            rotation,
        }
    }

    /// Top-left pixel position of this block, relative to the ship origin.
    pub fn position_pixel(&self) -> Vec2 {
        self.position.as_vec2() * crate::constant::GRID_SIZE
    }
}

/// One live block in a `ShipGrid`.
#[derive(Clone, Debug)]
pub struct GridBlock {
    pub placement: BlockPlacement,
    pub entity: Entity,
}

/// Identity marker on a block entity. Placement data lives in `ShipGrid`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRef {
    pub id: Uuid,
}

/// Ownership marker on a block entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockOwner {
    pub ship: Entity,
}
