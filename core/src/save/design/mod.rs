pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

pub struct DesignSavePlugin;

impl Plugin for DesignSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DesignSaveRequested>()
            .add_message::<ShipSavesChanged>()
            .init_resource::<ShipCatalog>()
            .init_resource::<DesignCatalogSource>()
            .add_systems(PreStartup, init_ship_catalog)
            .add_systems(
                Update,
                persist_design_request.run_if(on_message::<DesignSaveRequested>),
            );
    }
}
