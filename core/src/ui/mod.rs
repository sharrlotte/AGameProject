use bevy::prelude::*;

use crate::simulation::SimulationSet;

pub mod modal;
pub mod state;

pub use modal::*;
pub use state::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModalStack>()
            .add_systems(OnExit(SceneState::Game), clear_modals_on_scene_exit)
            .add_systems(
                Update,
                sync_modal_time
                    .run_if(resource_changed::<ModalStack>)
                    .in_set(SimulationSet::Present),
            );
    }
}
