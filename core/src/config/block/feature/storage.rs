use std::num::NonZeroU32;

use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub capacity: NonZeroU32,
}
