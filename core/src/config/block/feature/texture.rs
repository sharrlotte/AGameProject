use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::sprite::Texture;
use crate::validate::{FiniteF32, FiniteVec2, NonEmpty};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextureConfig {
    pub image: NonEmpty<String>,
    pub offset: FiniteVec2,
    pub z: FiniteF32,
    pub animated: bool,
}

impl TextureConfig {
    pub fn into_feature(self, assets: &AssetServer) -> Texture {
        Texture {
            image: assets.load(format!("texture/{}", self.image.get())),
            offset: self.offset.get(),
            z: self.z.get(),
            animated: self.animated,
        }
    }
}
