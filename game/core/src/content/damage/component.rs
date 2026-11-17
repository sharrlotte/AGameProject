use bevy::prelude::*;

/// One reusable floating damage-number entity.
#[derive(Component, Debug)]
pub struct DamageTextSlot {
    pub remaining: f32,
}

/// Round-robin pool of damage-number entities, avoiding per-hit spawn churn.
#[derive(Resource)]
pub struct DamageTextPool {
    pub slots: Vec<Entity>,
    pub next: usize,
}

#[derive(Component, Debug)]
pub struct Damage {
    pub damage: f32,
}

#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}
