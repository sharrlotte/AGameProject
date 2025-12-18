use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum ShootingProjectile {
    Bullet {
        speed: f32,
        damage: f32,
        lifetime: f32,
        piercing: u8,
    },
    Beam {
        range: f32,
        damage_per_second: f32,
        width: f32,
    },
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum ShootPattern {
    Single {
        fire_rate: f32,
        last_fired_at: f32,
        projectile: ShootingProjectile,
    },
    Burst {
        count: u32,
        delay: f32,
        current_shot: u32,
        burst_timer: f32,
        is_bursting: bool,
        projectile: ShootingProjectile,
    },
    Continuous,
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TurretController {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub fire_all_at_once: bool,
    pub shooting_pattern: ShootPattern,

    #[reflect(ignore)]
    pub potential_targets: HashSet<Entity>,

    pub active: bool,
    pub target_pos: Option<Vec2>,
    pub current_barrel_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TurretFeature {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub fire_all_at_once: bool,
    pub shooting_pattern: ShootPattern,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct PlayerFireController;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AIFireController;
