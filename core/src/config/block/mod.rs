pub mod def;
pub mod feature;

use std::collections::HashMap;

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::content::block::BlockRegistry;

pub use def::BlockDefConfig;
#[allow(unused_imports)]
pub use feature::{
    AmmoConfig, BatteryConfig, BoosterConfig, BurstShootPatternConfig, ConsumerConfig,
    FeatureConfig, GeneratorConfig, ShieldConfig, ShieldShapeConfig, ShootPatternConfig,
    StorageConfig, TextureConfig, ThrusterConfig, TurretChargeConfig, TurretComponentConfig,
    TurretDrainConfig, TurretPowerConfig,
};

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(try_from = "HashMap<String, BlockDefConfig>")]
pub struct Blocks(HashMap<String, BlockDefConfig>);

impl Blocks {
    pub fn as_map(&self) -> &HashMap<String, BlockDefConfig> {
        &self.0
    }
}

impl TryFrom<HashMap<String, BlockDefConfig>> for Blocks {
    type Error = String;

    fn try_from(blocks: HashMap<String, BlockDefConfig>) -> Result<Self, Self::Error> {
        if !blocks.values().any(|b| b.block_group == "core") {
            return Err("block configuration must define at least one core block".to_string());
        }
        Ok(Self(blocks))
    }
}

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlocksConfig {
    pub blocks: Blocks,
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
        for (key, block) in blocks.blocks.as_map() {
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
