pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::config::ConfigState;
use crate::system::load_default_fleet;

pub struct DesignSavePlugin;

impl Plugin for DesignSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DesignSaveRequested>()
            .add_message::<ShipSavesChanged>()
            .init_resource::<ShipCatalog>()
            .init_resource::<DesignCatalogSource>()
            .add_systems(
                OnEnter(ConfigState::Ready),
                init_ship_catalog.before(load_default_fleet),
            )
            .add_systems(
                Update,
                persist_design_request.run_if(on_message::<DesignSaveRequested>),
            );
    }
}
