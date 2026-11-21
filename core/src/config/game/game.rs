use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::validate::{Validate, check};
use super::fleet::{default_fleet_spacing, validate_fleet_spacing};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
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
    pub debris_lifetime_min_seconds: f32,
    pub debris_lifetime_max_seconds: f32,
    #[serde(default)]
    pub default_fleet: Vec<String>,
    #[serde(default = "default_fleet_spacing")]
    pub fleet_spacing: f32,
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
            debris_lifetime_min_seconds: 30.0,
            debris_lifetime_max_seconds: 60.0,
            default_fleet: Vec::new(),
            fleet_spacing: default_fleet_spacing(),
        }
    }
}

impl Validate for SimConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::positive(
            "minimum debris lifetime",
            self.debris_lifetime_min_seconds,
        )?;
        check::positive(
            "maximum debris lifetime",
            self.debris_lifetime_max_seconds,
        )?;
        check::range_ordered(
            "minimum debris lifetime",
            self.debris_lifetime_min_seconds,
            "maximum debris lifetime",
            self.debris_lifetime_max_seconds,
        )?;
        validate_fleet_spacing(self.fleet_spacing)?;
        Ok(())
    }
}

impl SimConfig {
    pub fn validate(&self) -> Result<(), String> {
        Validate::validate(self, &())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inverted_debris_lifetime_range() {
        let config = SimConfig {
            debris_lifetime_min_seconds: 60.0,
            debris_lifetime_max_seconds: 30.0,
            ..SimConfig::default()
        };

        let error = config.validate().expect_err("inverted range is invalid");
        assert!(error.contains("maximum debris lifetime"), "{error}");
    }

    #[test]
    fn rejects_non_positive_debris_lifetime() {
        let config = SimConfig {
            debris_lifetime_min_seconds: 0.0,
            ..SimConfig::default()
        };

        let error = config.validate().expect_err("zero lifetime is invalid");
        assert!(error.contains("must be positive"), "{error}");
    }

    #[test]
    fn accepts_valid_debris_lifetime_range() {
        assert!(SimConfig::default().validate().is_ok());
    }
}
