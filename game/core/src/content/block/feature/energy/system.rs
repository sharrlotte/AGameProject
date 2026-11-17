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
    mut ships: Query<(Entity, &mut ShipStats)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(production: f32, rates: &[f32]) -> (App, Entity) {
        let mut app = App::new();
        app.add_systems(Update, update_power_efficiency);

        let ship = app.world_mut().spawn(ShipStats::default()).id();
        if production > 0.0 {
            app.world_mut()
                .spawn((BlockOwner { ship }, PowerProducer { output: production }));
        }
        for rate in rates {
            app.world_mut().spawn((
                BlockOwner { ship },
                EnergyConsumer { rate: *rate },
                PowerEfficiency::default(),
            ));
        }
        (app, ship)
    }

    #[test]
    fn consumer_feature_inserts_its_runtime_component() {
        let mut app = App::new();
        app.add_systems(Update, update_power_efficiency);
        app.insert_resource(crate::config::GameConfigValue(
            crate::config::GameConfig::default(),
        ));

        let ship = app.world_mut().spawn(ShipStats::default()).id();
        let block = app.world_mut().spawn(()).id();
        app.world_mut()
            .entity_mut(block)
            .insert(BlockOwner { ship })
            .with_children(|_| {});
        // Insert exactly what `EnergyConsumer::insert` installs on a block.
        let consumer = EnergyConsumer { rate: 5.0 };
        consumer.insert(&mut app.world_mut().commands(), block);
        app.world_mut()
            .entity_mut(block)
            .insert(BlockOwner { ship });

        assert!(app.world().get::<EnergyConsumer>(block).is_some());
        app.update();
        assert_eq!(
            app.world().get::<ShipStats>(ship).unwrap().power_demand,
            5.0
        );
    }

    #[test]
    fn no_consumers_runs_at_full_efficiency() {
        let (mut app, ship) = app_with(10.0, &[]);
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.power_production, 10.0);
        assert_eq!(stats.power_demand, 0.0);
        assert_eq!(stats.power_used, 0.0);
        assert_eq!(stats.efficiency, 1.0);
    }

    #[test]
    fn surplus_production_runs_everything_at_full_efficiency() {
        let (mut app, ship) = app_with(20.0, &[5.0, 4.0]);
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.efficiency, 1.0);
        assert_eq!(stats.power_used, 9.0);
    }

    #[test]
    fn deficit_is_split_equally_not_by_demand() {
        let (mut app, ship) = app_with(6.0, &[4.0, 6.0]);
        app.update();

        let efficiencies: Vec<f32> = {
            let mut query = app.world_mut().query::<&PowerEfficiency>();
            query.iter(app.world()).map(|e| e.value()).collect()
        };
        assert_eq!(efficiencies.len(), 2);
        // share = 6 / 2 = 3; 3/4 = 0.75 and 3/6 = 0.5.
        assert!((efficiencies[0] - 0.75).abs() < 1e-5, "{efficiencies:?}");
        assert!((efficiencies[1] - 0.5).abs() < 1e-5, "{efficiencies:?}");

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert!((stats.power_used - 6.0).abs() < 1e-5);
        assert!((stats.efficiency - 0.6).abs() < 1e-5);
    }

    #[test]
    fn zero_production_disables_all_consumers() {
        let (mut app, ship) = app_with(0.0, &[5.0]);
        app.update();

        let mut query = app.world_mut().query::<&PowerEfficiency>();
        let efficiency = query.single(app.world()).unwrap().value();
        assert_eq!(efficiency, 0.0);

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.power_used, 0.0);
        assert_eq!(stats.efficiency, 0.0);
    }
}
