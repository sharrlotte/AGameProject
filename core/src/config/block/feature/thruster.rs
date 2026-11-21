use serde::Deserialize;

use crate::content::block::feature::thruster::Thruster;
use crate::validate::NonNegativeF32;

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ThrusterConfig {
    pub force: NonNegativeF32,
    pub torque: NonNegativeF32,
}

impl ThrusterConfig {
    pub fn into_feature(self) -> Thruster {
        Thruster::new(self.force.get(), self.torque.get())
    }
}
