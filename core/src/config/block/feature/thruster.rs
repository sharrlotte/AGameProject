use crate::content::block::feature::thruster::Thruster;
use crate::validate::{Validate, check};

pub use crate::content::block::feature::thruster::Thruster as ThrusterConfig;

impl Validate for Thruster {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("thruster force", self.force())
            .and_then(|_| check::non_negative("thruster torque", self.torque()))
            .map_err(|err| format!("block {block_type}: {err}"))
    }
}
