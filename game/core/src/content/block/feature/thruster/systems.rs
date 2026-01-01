use bevy::prelude::*;

use super::*;
use crate::content::block::{
    feature::efficiency::{BlockEfficiency, EfficiencyEffect},
    stat::StatsUpdateSet,
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

pub fn update_thrust_power_stats(
    mut query: Query<(
        &ThrustPowerBase,
        &mut ThrustPowerComputed,
        Option<&ThrustPowerEffects>,
        Option<&EfficiencyEffect>,
        Option<&BlockEfficiency>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect, block_efficiency) in &mut query {
        let eff = if let Some(be) = block_efficiency {
            be.value()
        } else {
            efficiency_effect.map(|e| e.value()).unwrap_or(1.0)
        };

        // Efficiency increases thrust.
        let mut val = base.value() * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value(), effect.value);
            }
        }
        computed.set(val);
    }
}

pub fn update_rotate_speed_stats(
    mut query: Query<(
        &RotateSpeedBase,
        &mut RotateSpeedComputed,
        Option<&RotateSpeedEffects>,
        Option<&EfficiencyEffect>,
        Option<&BlockEfficiency>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect, block_efficiency) in &mut query {
        let eff = if let Some(be) = block_efficiency {
            be.value()
        } else {
            efficiency_effect.map(|e| e.value()).unwrap_or(1.0)
        };

        // Efficiency increases rotate speed.
        let mut val = base.value() * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value(), effect.value);
            }
        }
        computed.set(val);
    }
}
