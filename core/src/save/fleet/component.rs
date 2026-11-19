use serde::{Deserialize, Serialize};

use crate::content::ship::ShipDesign;

pub const FLEET_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetSave {
    pub version: u32,
    pub ships: Vec<ShipDesign>,
}

crate::ron_document!(FleetSave, FLEET_SCHEMA_VERSION, "fleet");
