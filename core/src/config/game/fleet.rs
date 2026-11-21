use serde::Deserialize;

use crate::validate::NonEmpty;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DefaultFleetEntry {
    pub design: NonEmpty<String>,
}
