use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::content::block::base::BlockType;

#[derive(Component, Debug, Default, Reflect)]
pub struct Ship {
    pub id: usize,
    pub name: String,
    pub center: Vec2,
    pub thrust_power: f32,
    pub turn_speed: f32,
    pub radar_range: f32,
    pub blocks: Vec<BlockData>,
}

#[derive(Component)]
pub struct PlayerShipController;

#[derive(Component)]
pub struct AIShipController {
    pub behavior: AIShipBehavior,
    pub initial_behavior: AIShipBehavior,
}

#[derive(Debug, Clone, Copy)]
pub enum AIShipBehavior {
    Idle,
    Wander,
    Attack { target_pos: Vec2 },
    Patrol { around_pos: Vec2 },
    Chase { target: Entity },
    Flee { from: Entity },
    Guard { target: Entity },
    Follow { target: Entity },
}

#[derive(Component)]
pub struct RebuildShipCollider;

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

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct BlockData {
    pub id: usize,
    pub pos: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShipData {
    pub id: usize,
    pub name: String,
    pub blocks: Vec<BlockData>,
}
