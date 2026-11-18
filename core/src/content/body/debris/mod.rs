pub mod component;
mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct DebrisPlugin;

impl Plugin for DebrisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            explode_expired_debris
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Cleanup),
        );
    }
}
