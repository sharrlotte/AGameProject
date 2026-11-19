pub mod component;
pub mod lib;
pub mod system;

pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::config::ConfigState;
use crate::simulation::SimulationSet;

pub struct FleetSavePlugin;

impl Plugin for FleetSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            save_fleet_on_change
                .run_if(in_state(ConfigState::Ready))
                .in_set(SimulationSet::Present),
        );
    }
}
