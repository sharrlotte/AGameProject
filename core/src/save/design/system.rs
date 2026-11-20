use bevy::prelude::*;

use crate::save::design::component::{DesignCatalogSource, DesignSaveRequested, ShipCatalog};
use crate::save::design::lib::load_or_seed_designs;
use crate::save::world::ActiveWorldStore;

pub fn init_ship_catalog(mut catalog: ResMut<ShipCatalog>) {
    catalog.designs = load_or_seed_designs();
    catalog.source = DesignCatalogSource::HomeSandbox;
}

pub fn persist_design_request(
    mut events: MessageReader<DesignSaveRequested>,
    source: Res<DesignCatalogSource>,
    mut store: ResMut<ActiveWorldStore>,
    mut catalog: ResMut<ShipCatalog>,
) {
    for event in events.read() {
        let result: Result<(), String> = match *source {
            DesignCatalogSource::ActiveWorldDb => store
                .0
                .as_mut()
                .ok_or_else(|| "no active world store".to_string())
                .and_then(|active| {
                    active
                        .save_design(&event.design)
                        .map_err(|e| e.to_string())
                }),
            DesignCatalogSource::HomeSandbox => catalog
                .save_sandbox(&event.design)
                .map_err(|e| e.to_string()),
        };

        match result {
            Ok(()) => catalog.upsert(event.design.clone()),
            Err(e) => error!("Failed to save ship design: {e}"),
        }
    }
}
