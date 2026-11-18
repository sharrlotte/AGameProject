pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::content::block::feature::power::PowerSet;
use crate::content::block::stat::StatsUpdateSet;
use crate::simulation::SimulationSet;

pub struct ThrusterPlugin;

impl Plugin for ThrusterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            update_thruster_demand.in_set(PowerSet::Demand),
        )
        .add_systems(
            FixedPreUpdate,
            (update_thrust_power_stats, update_rotate_speed_stats)
                .in_set(StatsUpdateSet)
                .in_set(SimulationSet::Stats),
        );
    }
}
