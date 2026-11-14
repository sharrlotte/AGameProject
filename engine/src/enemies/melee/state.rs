use bevy::prelude::*;

use crate::config::MeleeSpec;

/// Wind-up melee capability driven by the archetype behavior list.
#[derive(Component, Debug, Clone, Copy)]
pub struct Melee {
    pub damage: f32,
    pub cooldown: f32,
    pub windup: f32,
    pub reach: f32,
    pub knockback: f32,
    pub cooldown_timer: f32,
    pub windup_timer: f32,
}

impl Melee {
    pub fn from_spec(spec: MeleeSpec) -> Self {
        Self {
            damage: spec.damage,
            cooldown: spec.cooldown,
            windup: spec.windup,
            reach: spec.reach,
            knockback: spec.knockback,
            cooldown_timer: 0.0,
            windup_timer: 0.0,
        }
    }

    pub fn sync(&mut self, spec: MeleeSpec) {
        self.damage = spec.damage;
        self.cooldown = spec.cooldown;
        self.windup = spec.windup;
        self.reach = spec.reach;
        self.knockback = spec.knockback;
    }

    pub fn is_winding(&self) -> bool {
        self.windup_timer > 0.0
    }
}
