pub mod system;

pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct ShipDebugPlugin;

impl Plugin for ShipDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            draw_ship_debug_gizmos
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        );
    }
}
