use bevy::prelude::*;

use crate::config::RangedSpec;

pub const ACID_PROJECTILE_RADIUS: f32 = 0.12;
pub const ACID_MAX_FLIGHT_SECS: f32 = 6.0;

/// Ranged acid capability driven by the archetype behavior list.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ranged {
    pub damage: f32,
    pub cooldown: f32,
    pub windup: f32,
    pub reach: f32,
    pub projectile_speed: f32,
    pub arc_deg: f32,
    pub pool_radius: f32,
    pub pool_lifetime: f32,
    pub cooldown_timer: f32,
    pub windup_timer: f32,
}

impl Ranged {
    pub fn from_spec(spec: RangedSpec) -> Self {
        Self {
            damage: spec.damage,
            cooldown: spec.cooldown,
            windup: spec.windup,
            reach: spec.reach,
            projectile_speed: spec.projectile_speed,
            arc_deg: spec.arc_deg,
            pool_radius: spec.pool_radius,
            pool_lifetime: spec.pool_lifetime,
            cooldown_timer: 0.0,
            windup_timer: 0.0,
        }
    }

    pub fn sync(&mut self, spec: RangedSpec) {
        self.damage = spec.damage;
        self.cooldown = spec.cooldown;
        self.windup = spec.windup;
        self.reach = spec.reach;
        self.projectile_speed = spec.projectile_speed;
        self.arc_deg = spec.arc_deg;
        self.pool_radius = spec.pool_radius;
        self.pool_lifetime = spec.pool_lifetime;
    }

    pub fn is_winding(&self) -> bool {
        self.windup_timer > 0.0
    }
}

/// Ballistic acid projectile that only interacts with world geometry.
#[derive(Component, Debug, Clone, Copy)]
pub struct AcidProjectile {
    pub damage: f32,
    pub pool_radius: f32,
    pub pool_lifetime: f32,
    pub lifetime: f32,
}
