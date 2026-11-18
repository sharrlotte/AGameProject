pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::scene::pause::handle_pause_input;
use crate::simulation::SimulationSet;

pub struct ShipDebugPlugin;

impl Plugin for ShipDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipDebugSelection>()
            .add_message::<ShipDebugSelected>()
            .add_message::<ShipDebugSelectionCleared>()
            .add_systems(
                Update,
                (
                    ignore_hud_picking,
                    clear_selection_on_escape.before(handle_pause_input),
                    apply_selection,
                    remove_despawned_ships,
                )
                    .chain()
                    .in_set(SimulationSet::Input)
                    .run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedUpdate,
                update_ship_ui
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                PostUpdate,
                draw_ship_debug_gizmos
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Present),
            );
    }
}
