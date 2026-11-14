use bevy::prelude::*;

use crate::config::DetonateSpec;

/// Proximity fuse owned by detonating archetypes.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fuse {
    pub trigger_radius: f32,
    pub fuse: f32,
    pub damage: f32,
    pub radius: f32,
    pub knockback: f32,
    pub timer: f32,
    pub armed: bool,
}

impl Fuse {
    pub fn from_spec(spec: DetonateSpec) -> Self {
        Self {
            trigger_radius: spec.trigger_radius,
            fuse: spec.fuse,
            damage: spec.damage,
            radius: spec.radius,
            knockback: spec.knockback,
            timer: 0.0,
            armed: false,
        }
    }

    pub fn sync(&mut self, spec: DetonateSpec) {
        self.trigger_radius = spec.trigger_radius;
        self.fuse = spec.fuse;
        self.damage = spec.damage;
        self.radius = spec.radius;
        self.knockback = spec.knockback;
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }
}

/// Ensures a detonating entity resolves at most one blast.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Detonated;
