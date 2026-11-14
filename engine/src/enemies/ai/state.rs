use bevy::prelude::*;
use crate::common::EnemyKind;

use crate::config::EnemyArchetypeData;

#[derive(Component, Debug, Clone, Copy)]
pub struct FacingDirection(pub Vec3);

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub move_speed: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
    pub scrap_value: u32,
    pub knockback_vel: Vec3,
}

impl Enemy {
    pub fn from_archetype(arch: &EnemyArchetypeData) -> Self {
        Self {
            kind: arch.kind,
            move_speed: arch.move_speed,
            radius: arch.radius,
            half_height: arch.half_height,
            mass: arch.mass,
            scrap_value: arch.scrap_value,
            knockback_vel: Vec3::ZERO,
        }
    }
}
