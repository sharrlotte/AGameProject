pub mod design;
pub mod document;
pub mod world;

use bevy::prelude::*;

use design::DesignSavePlugin;
use world::WorldSavePlugin;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DesignSavePlugin, WorldSavePlugin));
    }
}
