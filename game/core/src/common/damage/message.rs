use bevy::prelude::*;

#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}
