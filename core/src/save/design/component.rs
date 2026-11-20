use bevy::prelude::*;

use crate::content::ship::ShipDesign;
use crate::save::document::DocumentError;

pub const DESIGN_SCHEMA_VERSION: u32 = 3;

crate::ron_document!(ShipDesign, DESIGN_SCHEMA_VERSION, "ship design");

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignCatalogSource {
    #[default]
    HomeSandbox,
    ActiveWorldDb,
}

#[derive(Message)]
pub struct DesignSaveRequested {
    pub design: ShipDesign,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ShipCatalog {
    pub designs: Vec<ShipDesign>,
    pub source: DesignCatalogSource,
}

impl ShipCatalog {
    pub fn designs(&self) -> &[ShipDesign] {
        &self.designs
    }

    pub fn upsert(&mut self, design: ShipDesign) {
        let target_file = Self::file_name(&design);
        if let Some(existing) = self
            .designs
            .iter_mut()
            .find(|d| Self::file_name(d) == target_file)
        {
            *existing = design;
        } else {
            self.designs.push(design);
            self.designs.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    pub fn replace_all(&mut self, designs: Vec<ShipDesign>) {
        self.designs = designs;
        self.designs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn save_sandbox(&self, design: &ShipDesign) -> Result<(), DocumentError> {
        super::lib::save_design_file(design)
    }

    pub fn file_name(design: &ShipDesign) -> String {
        super::lib::design_file_name(design)
    }
}
