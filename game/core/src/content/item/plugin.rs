use bevy::prelude::*;

use crate::content::item::resource::{ItemRegistry, register_default_items};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(PreStartup, register_default_items);
    }
}
