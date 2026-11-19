pub mod lib;
pub mod system;

pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct VisionPlugin;

impl Plugin for VisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            sync_enemy_visibility
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        );
    }
}
