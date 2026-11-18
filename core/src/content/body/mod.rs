pub mod debris;
pub mod grid;
pub mod mutation;

pub use debris::*;
pub use grid::*;
pub use mutation::*;

use bevy::prelude::*;

use crate::content::block::handle_block_destroy;
use crate::simulation::SimulationSet;

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebrisPlugin)
            .add_message::<BodyMutation>()
            .add_systems(
                FixedPostUpdate,
                apply_body_mutations
                    .after(handle_block_destroy)
                    .in_set(SimulationSet::Death),
            );
    }
}
