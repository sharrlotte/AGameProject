use std::collections::HashSet;

use bevy::prelude::*;

pub mod emission;
pub mod targeting;
pub mod tracking;

use emission::handle_turret_emission;
use serde::{Deserialize, Serialize};
use targeting::{
    apply_targeting_to_fire_targeter, update_player_fire_controller, update_turret_target,
};

use crate::content::turret::tracking::{draw_turret_ranges, system_turret_tracking};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerFireController>()
            .register_type::<AIFireController>()
            .register_type::<FireStyle>()
            .register_type::<TurretBarrel>()
            .register_type::<TurretEmission>()
            .register_type::<TurretTargeter>()
            .add_systems(
                Update,
                (
                    update_player_fire_controller,
                    update_turret_target,
                    apply_targeting_to_fire_targeter,
                    handle_turret_emission,
                    system_turret_tracking,
                    draw_turret_ranges,
                ),
            );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TurretFeature {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TurretTargeter {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
    pub target: Option<Entity>,
    // Track potential targets currently inside the sensor range
    #[reflect(ignore)]
    pub potential_targets: HashSet<Entity>,

    pub active: bool,
    pub target_pos: Option<Vec2>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct PlayerFireController;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AIFireController;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TurretBarrel {
    pub barrel_offsets: Vec<Vec2>,
    pub current_barrel_index: usize,
    pub fire_all_at_once: bool,
}

impl Default for TurretBarrel {
    fn default() -> Self {
        Self {
            barrel_offsets: vec![Vec2::ZERO],
            current_barrel_index: 0,
            fire_all_at_once: false,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TurretEmission {
    pub emission_type: EmissionType,
    pub fire_rate: f32,
    pub timer: f32,
}

#[derive(Debug, Clone, Reflect)]
pub enum EmissionType {
    Bullet {
        speed: f32,
        damage: f32,
        lifetime: f32,
    },
    Beam {
        range: f32,
        damage_per_second: f32,
        width: f32,
    },
}

impl Default for TurretEmission {
    fn default() -> Self {
        Self {
            emission_type: EmissionType::Bullet {
                speed: 1000.0,
                damage: 10.0,
                lifetime: 2.0,
            },
            fire_rate: 0.5,
            timer: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub enum FireStyle {
    Single,
    Burst {
        count: u32,
        delay: f32,
        current_shot: u32,
        burst_timer: f32,
        is_bursting: bool,
    },
    Continuous,
}

impl Default for FireStyle {
    fn default() -> Self {
        Self::Single
    }
}
