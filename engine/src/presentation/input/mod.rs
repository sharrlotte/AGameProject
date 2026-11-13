use bevy::prelude::*;
use crate::common::GameState;
use crate::common::SimUpdateSet;

pub mod system;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            system::gather_local_input
                .before(SimUpdateSet)
                .run_if(in_state(GameState::Playing)),
        );
    }
}







