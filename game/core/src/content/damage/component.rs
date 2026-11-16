use bevy::prelude::*;
use crate::save::io::Save;

#[derive(Component, Debug)]
#[require(Save)]
pub struct Damage {
    pub damage: f32,
}

#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}
