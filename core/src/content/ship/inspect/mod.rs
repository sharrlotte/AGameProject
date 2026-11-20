pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::scene::pause::handle_modal_input;
use crate::simulation::SimulationSet;

pub struct ShipInspectPlugin;

impl Plugin for ShipInspectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectedShip>()
            .add_message::<ShipSelected>()
            .add_message::<ShipSelectionCleared>()
            .add_systems(
                Update,
                (
                    ignore_hud_picking,
                    clear_selection_on_escape.before(handle_modal_input),
                    apply_selection,
                    remove_despawned_ships,
                )
                    .chain()
                    .in_set(SimulationSet::Input)
                    .run_if(in_state(SceneState::Game)),
            );
    }
}
