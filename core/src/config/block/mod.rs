pub mod def;
pub mod feature;

use std::collections::HashMap;

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::content::block::BlockRegistry;
use crate::validate::Validate;

pub use def::BlockDefConfig;
#[allow(unused_imports)]
pub use feature::{
    AmmoConfig, BatteryConfig, BoosterConfig, BurstShootPatternConfig, ConsumerConfig,
    FeatureConfig, GeneratorConfig, ShieldConfig, ShieldShapeConfig, ShootPatternConfig,
    StorageConfig, TextureConfig, ThrusterConfig, TurretChargeConfig, TurretComponentConfig,
    TurretDrainConfig, TurretPowerConfig,
};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlocksConfig {
    pub blocks: HashMap<String, BlockDefConfig>,
}

impl Validate for BlocksConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        let mut has_core = false;
        for (key, block) in &self.blocks {
            block.validate(key)?;
            if block.block_group == "core" {
                has_core = true;
            }
        }
        if !has_core {
            return Err("block configuration must define at least one core block".to_string());
        }
        Ok(())
    }
}

pub fn blocks_bridge(
    mut events: MessageReader<AssetEvent<BlocksConfig>>,
    block_assets: Res<Assets<BlocksConfig>>,
    assets: Res<AssetServer>,
    mut registry: ResMut<BlockRegistry>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        let Some(blocks) = block_assets.get(id) else {
            continue;
        };

        let mut next = BlockRegistry::default();
        let mut ok = true;
        for (key, block) in &blocks.blocks {
            match block.clone().into_def(assets.as_ref(), key.clone()) {
                Ok(def) => next.register(key, def),
                Err(error) => {
                    error!("hot-reload block {key} failed: {error}");
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            *registry = next;
        }
    }
}
