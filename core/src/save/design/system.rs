use bevy::prelude::*;

use super::component::ShipCatalog;
use super::lib::load_or_seed_designs;

pub fn init_ship_catalog(mut catalog: ResMut<ShipCatalog>) {
    catalog.designs = load_or_seed_designs();
}
