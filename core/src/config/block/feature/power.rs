use crate::content::block::feature::power::{Battery, Booster, Consumer, Generator};
use crate::validate::{Validate, check};

pub use crate::content::block::feature::power::{
    Battery as BatteryConfig, Booster as BoosterConfig, Consumer as ConsumerConfig,
    Generator as GeneratorConfig,
};

impl Validate for Generator {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("generator power output", self.output)
            .map_err(|err| format!("block {block_type}: {err}"))
    }
}

impl Validate for Battery {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("battery storage capacity", self.capacity)
            .map_err(|err| format!("block {block_type}: {err}"))
    }
}

impl Validate for Consumer {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("consumer power consumption rate", self.rate)
            .map_err(|err| format!("block {block_type}: {err}"))
    }
}

impl Validate for Booster {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("booster effect radius", self.radius)
            .and_then(|_| check::non_negative("booster power consumption rate", self.rate))
            .and_then(|_| check::at_least("booster amplification multiplier", self.boost, 1.0))
            .map_err(|err| format!("block {block_type}: {err}"))
    }
}
