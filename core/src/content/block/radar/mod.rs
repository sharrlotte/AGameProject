pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;

pub struct RadarPlugin;

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(reset_disabled_radar)
            .add_systems(Update, radar_scan.in_set(SimulationSet::Weapons));
    }
}
