use bevy::prelude::*;
use std::collections::HashMap;

use super::component::{EnergyConsumer, PowerEfficiency, PowerProducer};
use crate::content::{
    block::{
        BlockOwner,
        feature::efficiency::{BlockEfficiency, EfficiencyUpdateSet},
    },
    ship::stat::ShipStats,
};
use crate::simulation::SimulationSet;

/// Per-ship power totals gathered in the first pass.
#[derive(Default, Clone, Copy)]
struct ShipPower {
    production: f32,
    demand: f32,
    consumers: usize,
}

pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            update_power_efficiency
                .after(EfficiencyUpdateSet)
                .in_set(SimulationSet::Stats),
        );
    }
}

/// Equal-share power solver.
///
/// Producers create power from nothing. Consumers split the available
/// production equally: `share = production / consumer_count`, and each
/// consumer runs at `clamp(share / its_demand, 0, 1)`. A ship with no consumers
/// or enough production runs everything at full efficiency.
pub fn update_power_efficiency(
    mut ships: Query<&mut ShipStats>,
    mut blocks: Query<(
        &BlockOwner,
        Option<&EnergyConsumer>,
        Option<&PowerProducer>,
        Option<&mut PowerEfficiency>,
        Option<&mut BlockEfficiency>,
    )>,
) {
    let mut power = HashMap::<Entity, ShipPower>::new();

    for (owner, consumer, producer, _, _) in &blocks {
        let entry = power.entry(owner.ship).or_default();
        if let Some(producer) = producer {
            entry.production += producer.output.max(0.0);
        }
        if let Some(consumer) = consumer
            && consumer.rate > 0.0
        {
            entry.demand += consumer.rate;
            entry.consumers += 1;
        }
    }

    // Share is constant per ship; store it so the second pass can compute
    // each consumer's efficiency from its own demand.
    let mut shares = HashMap::<Entity, f32>::new();
    for (entity, mut stats) in &mut ships {
        let totals = power.entry(entity).or_default();
        stats.power_production = totals.production;
        stats.power_demand = totals.demand;
        stats.power_used = 0.0;

        if totals.consumers == 0 || totals.demand <= 0.0 {
            shares.insert(entity, f32::INFINITY);
            stats.efficiency = 1.0;
        } else if totals.production >= totals.demand {
            shares.insert(entity, f32::INFINITY);
            stats.efficiency = 1.0;
        } else {
            let share = totals.production / totals.consumers as f32;
            shares.insert(entity, share);
            stats.efficiency = totals.production / totals.demand;
        }
    }

    let mut used = HashMap::<Entity, f32>::new();
    for (owner, consumer, _, mut power_eff, mut block_eff) in &mut blocks {
        let Some(consumer) = consumer else {
            continue;
        };
        if consumer.rate <= 0.0 {
            continue;
        }

        let share = shares.get(&owner.ship).copied().unwrap_or(0.0);
        let efficiency = if share.is_infinite() {
            1.0
        } else {
            (share / consumer.rate).clamp(0.0, 1.0)
        };

        if let Some(eff) = power_eff.as_mut() {
            eff.value = efficiency;
        }
        if let Some(eff) = block_eff.as_mut() {
            eff.apply_power(efficiency);
        }
        *used.entry(owner.ship).or_default() += consumer.rate * efficiency;
    }

    for (entity, mut stats) in &mut ships {
        stats.power_used = used.get(&entity).copied().unwrap_or(0.0);
    }
}
