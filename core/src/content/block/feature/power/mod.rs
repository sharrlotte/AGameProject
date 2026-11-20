pub mod booster;
pub mod component;
pub mod field;
pub mod grid;

pub use component::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;

pub const STARVATION_THRESHOLD: f32 = 0.01;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerSet {
    Generate,
    Boost,
    Demand,
    Solve,
}

pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(field::FieldPlugin)
            .register_type::<Supply>()
            .register_type::<Boost>()
            .register_type::<Demand>()
            .register_type::<Generator>()
            .register_type::<Battery>()
            .register_type::<Charge>()
            .register_type::<Consumer>()
            .register_type::<Booster>()
            .register_type::<BoosterState>()
            .init_resource::<grid::GridScratch>()
            .configure_sets(
                FixedFirst,
                (
                    PowerSet::Generate,
                    PowerSet::Boost,
                    PowerSet::Demand,
                    PowerSet::Solve,
                )
                    .chain()
                    .in_set(SimulationSet::Stats),
            )
            .add_systems(
                FixedFirst,
                grid::gather_generation.in_set(PowerSet::Generate),
            )
            .add_systems(
                FixedFirst,
                (booster::power_boosters, booster::apply_boosts)
                    .chain()
                    .in_set(PowerSet::Boost),
            )
            .add_observer(booster::reset_disabled_booster)
            .add_observer(grid::reset_disabled_demand)
            .add_systems(FixedFirst, grid::solve_power_grid.in_set(PowerSet::Solve));
    }
}
