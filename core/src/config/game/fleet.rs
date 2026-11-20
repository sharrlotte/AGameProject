use serde::Deserialize;

use crate::validate::{Validate, check};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DefaultFleetEntry {
    pub design: String,
}

impl Validate for DefaultFleetEntry {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        if self.design.trim().is_empty() {
            Err("default fleet entry design name must not be empty".to_string())
        } else {
            Ok(())
        }
    }
}

pub fn default_fleet_spacing() -> f32 {
    400.0
}

pub fn validate_fleet_spacing(spacing: f32) -> Result<(), String> {
    check::non_negative("fleet formation spacing", spacing)
}
