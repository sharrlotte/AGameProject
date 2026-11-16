use bevy::prelude::*;

use super::*;
use crate::content::block::{
    feature::efficiency::{BlockEfficiency, EfficiencyEffect},
    stat::StatsUpdateSet,
    stat::make_stat_system,
};

pub struct ThrusterPlugin;

impl Plugin for ThrusterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (update_thrust_power_stats, update_rotate_speed_stats).in_set(StatsUpdateSet),
        );
    }
}

make_stat_system!(update_thrust_power_stats, ThrustPowerBase, ThrustPowerComputed, ThrustPowerEffects, |base, eff| base * eff);
make_stat_system!(update_rotate_speed_stats, RotateSpeedBase, RotateSpeedComputed, RotateSpeedEffects, |base, eff| base * eff);
