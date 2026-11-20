use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::component::{BakedShape, BlockDef, Loot, ShapeError, ShapeSpec};
use crate::validate::{Validate, check};
use super::feature::FeatureConfig;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlockDefConfig {
    pub name: String,
    pub block_group: String,
    pub mass: f32,
    pub health: f32,
    pub cost: u32,
    pub shape: ShapeSpec,
    pub features: Vec<FeatureConfig>,
    pub drops: Vec<Loot>,
}

impl Validate for BlockDefConfig {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::positive("block mass", self.mass)?;
        check::positive("block health", self.health)?;
        for feature in &self.features {
            feature.validate(block_type)?;
        }
        Ok(())
    }
}

impl BlockDefConfig {
    pub fn into_def(
        self,
        assets: &AssetServer,
        block_type: String,
    ) -> Result<BlockDef, ShapeError> {
        let shape = BakedShape::try_from(&self.shape)
            .map_err(|error| ShapeError::new(format!("block {block_type}: {error}")))?;

        let features = self
            .features
            .into_iter()
            .map(|feature| feature.into_feature(assets, &block_type))
            .collect();

        Ok(BlockDef {
            name: self.name,
            block_type,
            block_group: self.block_group,
            mass: self.mass,
            health: self.health,
            cost: self.cost,
            shape,
            features,
            drops: self.drops,
        })
    }
}
