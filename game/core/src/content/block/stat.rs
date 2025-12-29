use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, plugin::ReadRapierContext, prelude::QueryFilter};

use crate::content::{
    block::base::{BlockOwner, PowerProducer},
    ship::stat::ComputedShipStats,
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EfficiencyUpdateSet;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            (
                apply_efficiency_buffs,
                update_block_efficiency_base,
                update_power_usage_stats,
                update_efficiency_base_on_power_usage,
            )
                .in_set(EfficiencyUpdateSet)
                .chain(),
        )
        .add_systems(
            FixedFirst,
            (update_thrust_power_stats, update_rotate_speed_stats).after(EfficiencyUpdateSet),
        );
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub enum StackRule {
    Additive,
    Multiply,
    Max,
    Min,
    Replace,
}

impl StackRule {
    pub fn apply(&self, current: f32, _base: f32, value: f32) -> f32 {
        match self {
            StackRule::Additive => current + value,
            StackRule::Multiply => current * value,
            StackRule::Max => current.max(value),
            StackRule::Min => current.min(value),
            StackRule::Replace => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct StatEffect {
    pub value: f32,
    pub rule: StackRule,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct EfficiencyEffect {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct BlockEfficiency {
    pub value: f32,
    pub base: f32,
}

impl Default for BlockEfficiency {
    fn default() -> Self {
        Self {
            value: 1.0,
            base: 1.0,
        }
    }
}

/// Component that provides efficiency buff to nearby blocks.
#[derive(Component, Clone, Copy, Debug, Reflect, Default, PartialEq)]
pub struct EfficiencyProvider {
    pub radius: f32,
    pub value: f32,
}

// PowerUsage
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct PowerUsageEffects(pub Vec<StatEffect>);

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

pub fn update_power_usage_stats(
    mut query: Query<(
        &PowerUsageBase,
        &mut PowerUsageComputed,
        Option<&PowerUsageEffects>,
        Option<&EfficiencyEffect>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect) in &mut query {
        let eff = efficiency_effect.map(|e| e.value).unwrap_or(1.0);

        let mut val = if eff > 0.0001 {
            base.value / eff
        } else {
            base.value
        };

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
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

pub fn update_efficiency_base_on_power_usage(
    mut ships: Query<(Entity, &mut ComputedShipStats)>,
    blocks: Query<(
        &BlockOwner,
        Option<&PowerUsageComputed>,
        Option<&PowerProducer>,
    )>,
) {
    // Reset ship stats
    for (_, mut stats) in &mut ships {
        stats.power_usage = 0.0;
    }

    let mut power_map = HashMap::<Entity, (f32, f32)>::new();

    for (owner, usage, producer) in &blocks {
        let entry = power_map.entry(owner.id).or_insert((0.0, 0.0));
        if let Some(u) = usage {
            entry.0 += u.value;
        }
        if let Some(p) = producer {
            entry.1 += p.output;
        }
    }

    for (entity, mut stats) in &mut ships {
        if let Some((usage, production)) = power_map.get(&entity) {
            stats.power_usage = *usage;

            if *usage > 0.0001 {
                if *production >= *usage {
                    stats.efficiency = 1.0;
                } else {
                    stats.efficiency = *production / *usage;
                }
            } else {
                stats.efficiency = 1.0;
            }
        }
    }
}

pub fn update_block_efficiency_base(
    mut blocks: Query<(&BlockOwner, &mut BlockEfficiency)>,
    ships: Query<&ComputedShipStats>,
) {
    for (owner, mut efficiency) in &mut blocks {
        if let Ok(ship_stats) = ships.get(owner.id) {
            efficiency.base = ship_stats.efficiency;
        } else {
            efficiency.base = 1.0;
        }
        // Reset value to base before applying buffs
        efficiency.value = efficiency.base;
    }
}

/// Applies spatial efficiency buffs.
pub fn apply_efficiency_buffs(
    mut blocks: Query<(Entity, &GlobalTransform, &mut BlockEfficiency)>,
    providers: Query<(&GlobalTransform, &EfficiencyProvider)>,
    rapier_context: ReadRapierContext,
) {
    let rapier = rapier_context.single().expect("Rapier context not found");
    let filter = QueryFilter::default().exclude_sensors();

    for (transform, provider) in &providers {
        let shape = Ball::new(provider.radius);
        let origin = transform.translation().truncate();

        rapier.intersect_shape(origin, 0.0, &shape, filter, |entity| {
            if let Ok((_, _, mut efficiency)) = blocks.get_mut(entity) {
                let buffed = efficiency.base * provider.value;

                if buffed > efficiency.value {
                    efficiency.value = buffed;
                }

                true
            } else {
                true
            }
        });
    }
}
