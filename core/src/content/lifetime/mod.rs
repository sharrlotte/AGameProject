pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_lifetime.in_set(SimulationSet::Cleanup));
    }
}
