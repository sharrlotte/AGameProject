pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipIndex>().add_systems(
            FixedUpdate,
            (build_ship_index, evaluate_ai_rules, apply_ai_behaviors)
                .chain()
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Ai),
        );
    }
}
