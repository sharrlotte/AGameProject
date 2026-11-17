pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

pub struct DesignSavePlugin;

impl Plugin for DesignSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipCatalog>()
            .add_systems(PreStartup, init_ship_catalog);
    }
}
