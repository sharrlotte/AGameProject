pub mod component;
pub mod config;
pub mod system;

pub use config::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Game), setup_minimap)
            .add_systems(
                Update,
                sync_marker_pool
                    .run_if(in_state(SceneState::Game))
                    .run_if(resource_changed::<MinimapConfigValue>)
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                PostUpdate,
                update_minimap
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Present),
            );
    }
}
