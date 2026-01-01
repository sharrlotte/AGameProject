use bevy::prelude::*;
use std::collections::HashMap;

use super::*;
use crate::content::{
    block::{
        components::{BlockOwner, PowerProducer}, feature::efficiency::{EfficiencyEffect, EfficiencyUpdateSet},
    },
    ship::stat::ComputedShipStats,
};

pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            (
                update_power_usage_stats,
                update_efficiency_base_on_power_usage,
            )
                .chain()
                .after(EfficiencyUpdateSet),
        );
    }
}

pub fn update_power_usage_stats(
    mut query: Query<(
        &PowerUsageBase,
        &mut PowerUsageComputed,
        Option<&PowerUsageEffects>,
        Option<&EfficiencyEffect>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect) in &mut query {
        let eff = efficiency_effect.map(|e| e.value()).unwrap_or(1.0);

        let mut val = if eff > 0.0001 {
            base.value() / eff
        } else {
            base.value()
        };

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value(), effect.value);
            }
        }
        computed.set(val);
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
            entry.0 += u.value();
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
