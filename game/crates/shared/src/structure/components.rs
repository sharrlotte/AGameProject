use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component defining the base stats of any object (Ship or Block).
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct Mass(pub f32);

/// Component for blocks that consume energy (e.g., Thrusters, Weapons).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PowerConsumer {
    pub required_input: f32,
    pub active: bool,
}

/// Component for blocks that produce energy (e.g., Reactors, Solar Panels).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PowerProducer {
    pub output: f32,
}

/// Component for blocks that store energy (e.g., Batteries).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PowerStorage {
    pub capacity: f32,
    pub current: f32,
}

/// Component identifying a thruster block.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Thruster {
    pub force: f32,
    /// Direction the thruster pushes (relative to the block's orientation).
    pub direction: Vec2,
}

/// Component identifying a weapon block.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Weapon {
    pub damage: f32,
    pub cooldown: f32,
    pub range: f32,
}
