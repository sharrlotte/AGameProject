use serde::Deserialize;

use crate::content::block::feature::power::{Battery, Booster, Consumer, Generator};
use crate::validate::{BoostFactor, NonNegativeF32};

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct GeneratorConfig {
    pub output: NonNegativeF32,
}

impl GeneratorConfig {
    pub fn into_feature(self) -> Generator {
        Generator {
            output: self.output.get(),
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BatteryConfig {
    pub capacity: NonNegativeF32,
}

impl BatteryConfig {
    pub fn into_feature(self) -> Battery {
        Battery {
            capacity: self.capacity.get(),
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConsumerConfig {
    pub rate: NonNegativeF32,
}

impl ConsumerConfig {
    pub fn into_feature(self) -> Consumer {
        Consumer {
            rate: self.rate.get(),
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BoosterConfig {
    pub radius: NonNegativeF32,
    pub boost: BoostFactor,
    pub rate: NonNegativeF32,
}

impl BoosterConfig {
    pub fn into_feature(self) -> Booster {
        Booster {
            radius: self.radius.get(),
            boost: self.boost.get(),
            rate: self.rate.get(),
        }
    }
}
