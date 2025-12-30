use bevy::prelude::*;

use crate::{
    constant::{THRUST_POWER_FACTOR, TURN_SPEED_FACTOR},
    content::block::{
        component::efficiency::{BlockEfficiency, EfficiencyEffect},
        stat::{StatEffect, StatsUpdateSet},
    },
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

// ThrustPower
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct ThrustPowerEffects(pub Vec<StatEffect>);

#[derive(Component, Debug, Clone, Reflect)]
pub struct Thruster {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct ThrusterComponent {
    pub force: f32,
    pub torque: f32,
}

// RotateSpeed
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct RotateSpeedEffects(pub Vec<StatEffect>);

impl ThrusterComponent {
    pub fn spawn(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            ThrustPowerBase {
                value: self.force * THRUST_POWER_FACTOR,
            },
            ThrustPowerComputed::default(),
            RotateSpeedBase {
                value: self.torque * TURN_SPEED_FACTOR,
            },
            RotateSpeedComputed::default(),
            BlockEfficiency::default(),
        ));
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
            be.value
        } else {
            efficiency_effect.map(|e| e.value).unwrap_or(1.0)
        };

        // Efficiency increases thrust.
        let mut val = base.value * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
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
            be.value
        } else {
            efficiency_effect.map(|e| e.value).unwrap_or(1.0)
        };

        // Efficiency increases rotate speed.
        let mut val = base.value * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
    }
}
