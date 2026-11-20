pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;

use bevy::{
    prelude::*,
    sprite_render::Material2dPlugin,
};

pub struct FieldPlugin;

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<OverdriveMaterial>::default())
            .register_type::<Field>()
            .add_systems(Startup, system::setup_assets)
            .add_systems(
                Update,
                (
                    system::spawn_field_on_booster,
                    system::sync_field_visibility,
                ),
            );
    }
}
