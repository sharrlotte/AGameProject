pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(PreStartup, register_default_items);
    }
}
