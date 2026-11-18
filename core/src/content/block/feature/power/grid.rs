use std::collections::HashMap;

use bevy::prelude::*;

use super::component::{Battery, Boost, Charge, Demand, Generator, Supply};
use crate::content::{block::BlockOwner, ship::stat::ShipStats};

#[derive(Default, Clone, Copy)]
pub struct ShipPowerScratch {
    pub generation: f32,
    pub stored: f32,
    pub capacity: f32,
    pub booster_draw: f32,
    pub remaining_available: f32,
}

#[derive(Resource, Default)]
pub struct GridScratch {
    pub ships: HashMap<Entity, ShipPowerScratch>,
}

#[derive(Clone, Copy, Debug)]
pub struct BoosterInput {
    pub entity: Entity,
    pub distance_sq: f32,
    pub rate: f32,
}

pub fn allocate_boosters(
    mut available_power: f32,
    boosters: &[BoosterInput],
) -> (Vec<bool>, f32, f32) {
    let mut powered = Vec::with_capacity(boosters.len());
    let mut draw = 0.0;

    for booster in boosters {
        let rate = booster.rate.max(0.0);
        if available_power >= rate {
            available_power -= rate;
            draw += rate;
            powered.push(true);
        } else {
            powered.push(false);
        }
    }

    (powered, draw, available_power)
}

pub fn gather_generation(
    mut scratch: ResMut<GridScratch>,
    ships: Query<Entity, With<ShipStats>>,
    generators: Query<(&BlockOwner, &Generator, Option<&Boost>)>,
    batteries: Query<(&BlockOwner, &Battery, &Charge)>,
) {
    scratch.ships.clear();
    for ship in &ships {
        scratch.ships.insert(ship, ShipPowerScratch::default());
    }

    for (owner, generator, boost) in &generators {
        let entry = scratch.ships.entry(owner.ship).or_default();
        let multiplier = boost.map_or(1.0, Boost::value);
        entry.generation += generator.output.max(0.0) * multiplier;
    }

    for (owner, battery, charge) in &batteries {
        let entry = scratch.ships.entry(owner.ship).or_default();
        let cap = battery.capacity.max(0.0);
        entry.capacity += cap;
        entry.stored += charge.value.clamp(0.0, cap);
    }
}

pub fn solve_power_grid(
    time: Res<Time>,
    scratch: Res<GridScratch>,
    mut ships: Query<(Entity, &mut ShipStats)>,
    consumers: Query<(&BlockOwner, &Demand)>,
    mut supplies: Query<(&BlockOwner, &mut Supply)>,
    mut batteries: Query<(&BlockOwner, &Battery, &mut Charge)>,
) {
    let dt = time.delta_secs();

    let mut total_demand = HashMap::<Entity, f32>::new();
    for (owner, demand) in &consumers {
        *total_demand.entry(owner.ship).or_default() += demand.value.max(0.0);
    }

    let mut ship_supply = HashMap::<Entity, f32>::new();
    let mut ship_new_stored = HashMap::<Entity, (f32, f32)>::new();

    for (ship_entity, mut stats) in &mut ships {
        let entry = scratch
            .ships
            .get(&ship_entity)
            .copied()
            .unwrap_or_default();
        let demand = total_demand.get(&ship_entity).copied().unwrap_or(0.0);

        let supply = if demand <= 0.0 {
            1.0
        } else {
            (entry.remaining_available / demand).clamp(0.0, 1.0)
        };

        let net = entry.generation - entry.booster_draw - demand * supply;
        let new_stored = if dt > 0.0 {
            (entry.stored + net * dt).clamp(0.0, entry.capacity)
        } else {
            entry.stored
        };

        stats.power_production = entry.generation;
        stats.power_demand = entry.booster_draw + demand;
        stats.power_supply = supply;
        stats.power_stored = new_stored;
        stats.power_capacity = entry.capacity;

        ship_supply.insert(ship_entity, supply);
        ship_new_stored.insert(ship_entity, (new_stored, entry.capacity));
    }

    for (owner, mut supply) in &mut supplies {
        if let Some(&value) = ship_supply.get(&owner.ship) {
            supply.value = value;
        }
    }

    for (owner, battery, mut charge) in &mut batteries {
        let Some(&(new_stored, total_cap)) = ship_new_stored.get(&owner.ship) else {
            continue;
        };
        if total_cap > 0.0 {
            charge.value = new_stored * (battery.capacity.max(0.0) / total_cap);
        } else {
            charge.value = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GridInput<'a> {
        generation: f32,
        stored: f32,
        capacity: f32,
        demand: f32,
        dt: f32,
        boosters: &'a [BoosterInput],
    }

    struct GridOutput {
        powered_boosters: Vec<bool>,
        booster_draw: f32,
        supply: f32,
        new_stored: f32,
    }

    fn solve_ship_grid(input: GridInput<'_>) -> GridOutput {
        let generation = input.generation.max(0.0);
        let capacity = input.capacity.max(0.0);
        let stored = input.stored.clamp(0.0, capacity);
        let demand = input.demand.max(0.0);

        let available_power = if input.dt > 0.0 {
            generation + stored / input.dt
        } else {
            generation
        };

        let (powered_boosters, booster_draw, remaining_available) =
            allocate_boosters(available_power, input.boosters);

        let supply = if demand <= 0.0 {
            1.0
        } else {
            (remaining_available / demand).clamp(0.0, 1.0)
        };

        let net = generation - booster_draw - demand * supply;
        let new_stored = if input.dt > 0.0 {
            (stored + net * input.dt).clamp(0.0, capacity)
        } else {
            stored
        };

        GridOutput {
            powered_boosters,
            booster_draw,
            supply,
            new_stored,
        }
    }

    #[test]
    fn surplus_supplies_everyone_fully() {
        let out = solve_ship_grid(GridInput {
            generation: 20.0,
            stored: 0.0,
            capacity: 0.0,
            demand: 9.0,
            dt: 1.0,
            boosters: &[],
        });
        assert_eq!(out.supply, 1.0);
        assert_eq!(out.new_stored, 0.0);
    }

    #[test]
    fn deficit_splits_proportionally_without_waste() {
        let out = solve_ship_grid(GridInput {
            generation: 9.0,
            stored: 0.0,
            capacity: 0.0,
            demand: 11.0,
            dt: 1.0,
            boosters: &[],
        });
        assert!((out.supply - 9.0 / 11.0).abs() < 1e-6);
    }

    #[test]
    fn no_demand_gives_full_supply() {
        let out = solve_ship_grid(GridInput {
            generation: 0.0,
            stored: 0.0,
            capacity: 0.0,
            demand: 0.0,
            dt: 1.0,
            boosters: &[],
        });
        assert_eq!(out.supply, 1.0);
    }

    #[test]
    fn surplus_charges_battery_clamped_to_capacity() {
        let out = solve_ship_grid(GridInput {
            generation: 15.0,
            stored: 98.0,
            capacity: 100.0,
            demand: 10.0,
            dt: 1.0,
            boosters: &[],
        });
        assert_eq!(out.supply, 1.0);
        assert_eq!(out.new_stored, 100.0);
    }

    #[test]
    fn battery_covers_deficit_and_runs_dry() {
        let covered = solve_ship_grid(GridInput {
            generation: 6.0,
            stored: 10.0,
            capacity: 100.0,
            demand: 10.0,
            dt: 1.0,
            boosters: &[],
        });
        assert_eq!(covered.supply, 1.0);
        assert!((covered.new_stored - 6.0).abs() < 1e-6);

        let dry = solve_ship_grid(GridInput {
            generation: 6.0,
            stored: 0.0,
            capacity: 100.0,
            demand: 10.0,
            dt: 1.0,
            boosters: &[],
        });
        assert!((dry.supply - 0.6).abs() < 1e-6);
        assert_eq!(dry.new_stored, 0.0);
    }

    #[test]
    fn booster_priority_powers_closer_first_and_can_starve_consumers() {
        let b1 = Entity::from_raw_u32(1).unwrap();
        let b2 = Entity::from_raw_u32(2).unwrap();
        let boosters = [
            BoosterInput {
                entity: b1,
                distance_sq: 1.0,
                rate: 6.0,
            },
            BoosterInput {
                entity: b2,
                distance_sq: 4.0,
                rate: 6.0,
            },
        ];

        let partial = solve_ship_grid(GridInput {
            generation: 10.0,
            stored: 0.0,
            capacity: 0.0,
            demand: 8.0,
            dt: 1.0,
            boosters: &boosters,
        });
        assert_eq!(partial.powered_boosters, vec![true, false]);
        assert_eq!(partial.booster_draw, 6.0);
        assert!((partial.supply - 0.5).abs() < 1e-6);

        let exact = solve_ship_grid(GridInput {
            generation: 6.0,
            stored: 0.0,
            capacity: 0.0,
            demand: 8.0,
            dt: 1.0,
            boosters: &[boosters[0]],
        });
        assert_eq!(exact.powered_boosters, vec![true]);
        assert_eq!(exact.supply, 0.0);
    }

    #[test]
    fn battery_lifecycle_and_two_ships_isolated() {
        let mut app = App::new();
        app.init_resource::<GridScratch>();
        app.insert_resource(Time::<()>::default());
        app.add_systems(
            Update,
            (gather_generation, super::super::booster::power_boosters, solve_power_grid).chain(),
        );

        let ship_a = app.world_mut().spawn(ShipStats::default()).id();
        let ship_b = app.world_mut().spawn(ShipStats::default()).id();

        app.world_mut()
            .spawn((BlockOwner { ship: ship_a }, Generator { output: 50.0 }));
        let mut cmds = app.world_mut().commands();
        let bat1 = cmds.spawn(BlockOwner { ship: ship_a }).id();
        Battery { capacity: 50.0 }.insert(&mut cmds, bat1);
        let bat2 = cmds.spawn(BlockOwner { ship: ship_a }).id();
        Battery { capacity: 50.0 }.insert(&mut cmds, bat2);
        app.world_mut().flush();

        app.world_mut()
            .spawn((BlockOwner { ship: ship_b }, Generator { output: 5.0 }));
        app.world_mut().spawn((
            BlockOwner { ship: ship_b },
            Demand { value: 10.0 },
            Supply::default(),
        ));

        app.update();

        let stats_a = app.world().get::<ShipStats>(ship_a).unwrap();
        assert_eq!(stats_a.power_capacity, 100.0);
        assert_eq!(stats_a.power_stored, 100.0);
        assert_eq!(stats_a.power_supply, 1.0);

        let stats_b = app.world().get::<ShipStats>(ship_b).unwrap();
        assert_eq!(stats_b.power_supply, 0.5);

        app.world_mut().despawn(bat1);
        app.update();
        let stats_a = app.world().get::<ShipStats>(ship_a).unwrap();
        assert_eq!(stats_a.power_capacity, 50.0);
        assert_eq!(stats_a.power_stored, 50.0);
    }

    #[test]
    fn generator_boost_lags_one_tick() {
        let mut app = App::new();
        app.init_resource::<GridScratch>();
        app.insert_resource(Time::<()>::default());
        app.add_systems(
            Update,
            (gather_generation, super::super::booster::power_boosters, solve_power_grid).chain(),
        );

        let ship = app.world_mut().spawn(ShipStats::default()).id();
        let generator = app
            .world_mut()
            .spawn((
                BlockOwner { ship },
                Generator { output: 25.0 },
                Boost { value: 1.0 },
            ))
            .id();

        app.update();
        assert_eq!(app.world().get::<ShipStats>(ship).unwrap().power_production, 25.0);

        // Boost becomes 1.5 during tick N; generation reflects it on tick N+1.
        app.world_mut().get_mut::<Boost>(generator).unwrap().value = 1.5;
        app.update();
        assert_eq!(app.world().get::<ShipStats>(ship).unwrap().power_production, 37.5);
    }
}
