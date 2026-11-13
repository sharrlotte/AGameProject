use bevy::prelude::*;
use crate::common::EnemyKind;

use crate::config::EnemyArchetypeData;

#[derive(Component, Debug, Clone, Copy)]
pub struct FacingDirection(pub Vec3);

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
    pub knockback_vel: Vec3,
}

impl Enemy {
    pub fn from_archetype(kind: EnemyKind, arch: EnemyArchetypeData) -> Self {
        Self {
            kind,
            move_speed: arch.move_speed,
            attack_damage: arch.attack_damage,
            attack_cooldown: arch.attack_cooldown,
            attack_timer: 0.0,
            radius: arch.radius,
            half_height: arch.half_height,
            mass: arch.mass,
            knockback_vel: Vec3::ZERO,
        }
    }
}

