use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::validate::{NonNegativeF32, OrderedRange, PositiveF32};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct SimConfig {
    pub thrust_power_factor: f32,
    pub turn_speed_factor: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub drop_size: f32,
    pub drop_despawn_in_seconds: f32,
    pub drop_collect_radius: f32,
    pub max_ships: usize,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub missile_accel_time: f32,
    pub missile_max_turn: f32,
    pub bullet_fade_time: f32,
    pub ai_flee_distance: f32,
    pub ai_wander_patrol_radius_multiplier: f32,
    pub ai_group_range_multiplier: f32,
    pub ai_min_range_factor: f32,
    pub ai_wander_attack_chance: f32,
    pub ai_wander_guard_chance: f32,
    pub auto_save_interval_secs: f32,
    pub debris_lifetime: OrderedRange<PositiveF32>,
    pub default_fleet: Vec<String>,
    pub fleet_spacing: NonNegativeF32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            thrust_power_factor: 50.0,
            turn_speed_factor: 9000.0,
            linear_damping: 0.9,
            angular_damping: 0.999,
            drop_size: 10.0,
            drop_despawn_in_seconds: 300.0,
            drop_collect_radius: 1000.0,
            max_ships: 16,
            min_zoom: 0.2,
            max_zoom: 30.0,
            missile_accel_time: 0.35,
            missile_max_turn: 2.0,
            bullet_fade_time: 0.25,
            ai_flee_distance: 1000.0,
            ai_wander_patrol_radius_multiplier: 10.0,
            ai_group_range_multiplier: 2.0,
            ai_min_range_factor: 0.9,
            ai_wander_attack_chance: 0.5,
            ai_wander_guard_chance: 0.5,
            auto_save_interval_secs: 30.0,
            debris_lifetime: OrderedRange::<PositiveF32>::from_const(30.0, 60.0),
            default_fleet: Vec::new(),
            fleet_spacing: NonNegativeF32::from_const(400.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inverted_debris_lifetime_range() {
        let ron = "(debris_lifetime: (60.0, 30.0))";
        let error = ron::from_str::<SimConfig>(ron).expect_err("inverted range is invalid");
        assert!(error.to_string().contains("must be <="), "{error}");
    }

    #[test]
    fn rejects_non_positive_debris_lifetime() {
        let ron = "(debris_lifetime: (0.0, 30.0))";
        let error = ron::from_str::<SimConfig>(ron).expect_err("zero lifetime is invalid");
        assert!(error.to_string().contains("finite and positive"), "{error}");
    }

    #[test]
    fn accepts_valid_debris_lifetime_range() {
        assert!(ron::from_str::<SimConfig>("()").is_ok());
    }
}
