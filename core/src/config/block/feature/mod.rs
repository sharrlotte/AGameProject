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

impl FeatureConfig {
    pub fn into_feature(self, assets: &AssetServer, block_type: &str) -> Feature {
        match self {
            FeatureConfig::Texture(config) => Feature::Texture(config.into_feature(assets)),
            FeatureConfig::Turret(config) => {
                Feature::Turret(config.into_turret(assets, block_type))
            }
            FeatureConfig::Generator(generator) => Feature::Generator(generator.into_feature()),
            FeatureConfig::Battery(battery) => Feature::Battery(battery.into_feature()),
            FeatureConfig::Consumer(consumer) => Feature::Consumer(consumer.into_feature()),
            FeatureConfig::Booster(booster) => Feature::Booster(booster.into_feature()),
            FeatureConfig::Thruster(thruster) => Feature::Thruster(thruster.into_feature()),
            FeatureConfig::Storage(config) => Feature::Storage(config.capacity.get()),
            FeatureConfig::Shield(config) => Feature::Shield(config.into_shield()),
        }
    }
}
