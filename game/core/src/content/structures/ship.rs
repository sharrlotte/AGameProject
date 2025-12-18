use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::content::structures::types::BlockType;

/// The main component for a spaceship entity.
/// It acts as the root for all attached blocks.
#[derive(Component, Debug, Default, Reflect)]
pub struct Ship {
    pub name: String,
    pub center: Vec2,
    pub thrust_power: f32,
    pub turn_speed: f32,
    pub radar_range: f32,
}

/// A spatial grid that maps local coordinates to Block Entities.
/// This allows for O(1) adjacency checks and easy validation.
#[derive(Component, Debug, Default, Reflect)]
pub struct ShipGrid {
    /// Maps (x, y) -> Entity ID
    #[reflect(ignore)]
    pub blocks: HashMap<IVec2, Entity>,
}

impl ShipGrid {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, pos: IVec2, entity: Entity) {
        self.blocks.insert(pos, entity);
    }

    pub fn get(&self, pos: IVec2) -> Option<&Entity> {
        self.blocks.get(&pos)
    }

    pub fn remove(&mut self, pos: IVec2) -> Option<Entity> {
        self.blocks.remove(&pos)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SavedBlock {
    pub pos: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SavedShip {
    pub name: String,
    pub blocks: Vec<SavedBlock>,
}

#[derive(Component)]
pub struct ShipBlockHitbox;
