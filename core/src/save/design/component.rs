use bevy::prelude::*;

use crate::content::ship::ShipDesign;
use crate::save::document::DocumentError;

use super::lib::save_design_file;

pub const DESIGN_SCHEMA_VERSION: u32 = 1;

crate::ron_document!(ShipDesign, DESIGN_SCHEMA_VERSION, "ship design");

#[derive(Resource, Default, Debug, Clone)]
pub struct ShipCatalog {
    pub designs: Vec<ShipDesign>,
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

    pub fn save_design(&mut self, design: &ShipDesign) -> Result<(), DocumentError> {
        save_design_file(design)?;
        self.upsert(design.clone());
        Ok(())
    }

    pub fn file_name(design: &ShipDesign) -> String {
        super::lib::design_file_name(design)
    }
}
