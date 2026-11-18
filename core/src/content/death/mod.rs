pub mod component;

pub use component::*;

use bevy::prelude::*;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShipDestroyed>();
    }
}
