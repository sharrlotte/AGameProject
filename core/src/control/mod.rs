pub mod component;
pub mod system;

pub use component::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;
use crate::ui::SceneState;
use system::{player_control_ship, toggle_control_mode};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlMode>()
            .init_resource::<CameraTarget>()
            .init_resource::<ControlledShip>()
            .add_systems(
                FixedUpdate,
                player_control_ship
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Control),
            )
            .add_systems(
                Update,
                toggle_control_mode
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Input),
            );
    }
}
