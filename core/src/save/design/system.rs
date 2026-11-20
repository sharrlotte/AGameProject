use bevy::prelude::*;

use crate::save::design::component::{
    DesignCatalogSource, DesignSaveRequested, ShipCatalog, ShipSavesChanged,
};
use crate::content::block::BlockRegistry;
use crate::save::design::lib::load_or_seed_designs;
use crate::save::world::WorldContext;

pub fn init_ship_catalog(mut catalog: ResMut<ShipCatalog>, registry: Res<BlockRegistry>) {
    catalog.designs = load_or_seed_designs(&registry);
    catalog.source = DesignCatalogSource::HomeSandbox;
}

pub fn persist_design_request(
    mut events: MessageReader<DesignSaveRequested>,
    source: Res<DesignCatalogSource>,
    context: Res<WorldContext>,
    mut catalog: ResMut<ShipCatalog>,
    mut ship_saved: MessageWriter<ShipSavesChanged>,
) {
    for event in events.read() {
        let result: Result<(), String> = match *source {
            DesignCatalogSource::ActiveWorldDb => context
                .store()
                .save_design(&event.design)
                .map_err(|e| e.to_string()),
            DesignCatalogSource::HomeSandbox => catalog
                .save_sandbox(&event.design)
                .map_err(|e| e.to_string()),
        };

        match result {
            Ok(()) => {
                catalog.upsert(event.design.clone());
                ship_saved.write(ShipSavesChanged);
            }
            Err(e) => error!("Failed to save ship design: {e}"),
        }
    }
}
