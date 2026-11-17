use bevy::prelude::*;
use std::collections::HashMap;

use super::*;
use crate::simulation::SimulationSet;
use crate::content::{
    block::{
        BlockOwner,
        component::PowerProducer,
        feature::efficiency::{BlockEfficiency, EfficiencyEffect, EfficiencyUpdateSet},
        stat::make_stat_system,
    },
    ship::stat::ShipStats,
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
                .after(EfficiencyUpdateSet)
                .in_set(SimulationSet::Stats),
        );
    }
}

make_stat_system!(update_power_usage_stats, PowerUsageBase, PowerUsageComputed, PowerUsageEffects, |base, eff| if eff > 0.0001 { base / eff } else { base });

pub fn update_efficiency_base_on_power_usage(
    mut ships: Query<(Entity, &mut ShipStats)>,
    blocks: Query<(
        &BlockOwner,
        Option<&PowerUsageComputed>,
        Option<&PowerProducer>,
    )>,
) {
    // Reset ship stats
    for (_, mut stats) in &mut ships {
        stats.power_used = 0.0;
        stats.power_production = 0.0;
        stats.efficiency = 1.0;
    }

    let mut power_map = HashMap::<Entity, (f32, f32)>::new();

    for (owner, usage, producer) in &blocks {
        let entry = power_map.entry(owner.ship).or_insert((0.0, 0.0));
        if let Some(u) = usage {
            entry.0 += u.value();
        }
        if let Some(p) = producer {
            entry.1 += p.output;
        }
    }

    for (entity, mut stats) in &mut ships {
        if let Some((usage, production)) = power_map.get(&entity) {
            stats.power_used = *usage;
            stats.power_production = *production;

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
