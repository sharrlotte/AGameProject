pub mod component;
pub mod system;

pub use component::*;

use bevy::prelude::*;

use crate::content::lifetime::update_lifetime;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChunkStreamError>()
            .init_resource::<StreamingTimer>()
            .add_systems(
                FixedUpdate,
                (
                    system::stream_chunks.after(update_lifetime),
                    system::on_chunk_stream_error,
                )
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Cleanup),
            );
    }
}
