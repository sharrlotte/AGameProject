use serde::Deserialize;

use crate::validate::{Validate, check};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub capacity: u32,
}

impl Validate for StorageConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::at_least("storage capacity", self.capacity, 1)
    }
}
