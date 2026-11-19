pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct ShipInspectorPlugin;

impl Plugin for ShipInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorBuildState>().add_systems(
            Update,
            (
                rebuild_inspector_diagram,
                refresh_inspector_ui.run_if(on_timer(Duration::from_millis(80))),
            )
                .chain()
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        );
    }
}
