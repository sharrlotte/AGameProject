pub mod ammo;
pub mod power;
pub mod shield;
pub mod storage;
pub mod texture;
pub mod thruster;
pub mod turret;

use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::component::Feature;
use crate::validate::Validate;

pub use ammo::AmmoConfig;
pub use power::{BatteryConfig, BoosterConfig, ConsumerConfig, GeneratorConfig};
pub use shield::{ShieldConfig, ShieldShapeConfig};
pub use storage::StorageConfig;
pub use texture::TextureConfig;
pub use thruster::ThrusterConfig;
pub use turret::{
    BurstShootPatternConfig, ShootPatternConfig, TurretChargeConfig, TurretComponentConfig,
    TurretDrainConfig, TurretPowerConfig,
};

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum FeatureConfig {
    Texture(TextureConfig),
    Turret(TurretComponentConfig),
    Generator(GeneratorConfig),
    Battery(BatteryConfig),
    Consumer(ConsumerConfig),
    Booster(BoosterConfig),
    Thruster(ThrusterConfig),
    Storage(StorageConfig),
    Shield(ShieldConfig),
}

impl Validate for FeatureConfig {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        match self {
            FeatureConfig::Texture(t) => t.validate(&()),
            FeatureConfig::Turret(t) => t.validate(block_type),
            FeatureConfig::Generator(g) => g.validate(block_type),
            FeatureConfig::Battery(b) => b.validate(block_type),
            FeatureConfig::Consumer(c) => c.validate(block_type),
            FeatureConfig::Booster(b) => b.validate(block_type),
            FeatureConfig::Thruster(th) => th.validate(block_type),
            FeatureConfig::Storage(s) => s.validate(&()),
            FeatureConfig::Shield(sh) => sh.validate(&()),
        }
    }
}

impl FeatureConfig {
    pub fn into_feature(self, assets: &AssetServer, block_type: &str) -> Feature {
        match self {
            FeatureConfig::Texture(config) => Feature::Texture(config.into_feature(assets)),
            FeatureConfig::Turret(config) => {
                Feature::Turret(config.into_turret(assets, block_type))
            }
            FeatureConfig::Generator(generator) => {
                Feature::Generator(generator.validated(block_type))
            }
            FeatureConfig::Battery(battery) => Feature::Battery(battery.validated(block_type)),
            FeatureConfig::Consumer(consumer) => Feature::Consumer(consumer.validated(block_type)),
            FeatureConfig::Booster(booster) => Feature::Booster(booster.validated(block_type)),
            FeatureConfig::Thruster(thruster) => Feature::Thruster(thruster.validated()),
            FeatureConfig::Storage(config) => Feature::Storage(config.capacity),
            FeatureConfig::Shield(config) => Feature::Shield(config.into_shield()),
        }
    }
}
