pub mod chunk;
pub mod event;

pub use event::SectorDiscovered;

use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SectorDiscovered>()
            .add_plugins(chunk::ChunkPlugin);
    }
}
