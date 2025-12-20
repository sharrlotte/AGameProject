use bevy::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;

use crate::{save::base::Savable, 
    world::base::BlockData}
;

#[derive(Component, Debug, Default)]
pub struct Ship {
    pub id: Uuid,
    pub center: Vec2,
    pub thrust_power: f32,
    pub turn_speed: f32,
    pub radar_range: f32,
    pub blocks: Vec<BlockData>,
}

pub struct ShipState {
    pub position: Vec2,
    pub rotation: f32,
    pub velocity: Vec2,
    pub angular_velocity: f32,
}

impl Savable for ShipState {}


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
    Attack { target_position: Vec2 },
    Patrol { around_position: Vec2 },
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

    pub fn insert(&mut self, position: IVec2, entity: Entity) {
        self.blocks.insert(position, entity);
    }

    pub fn get(&self, position: IVec2) -> Option<&Entity> {
        self.blocks.get(&position)
    }

    pub fn remove(&mut self, position: IVec2) -> Option<Entity> {
        self.blocks.remove(&position)
    }
}
