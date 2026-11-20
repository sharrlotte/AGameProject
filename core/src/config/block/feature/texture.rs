use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::sprite::Texture;
use crate::validate::{Validate, check};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextureConfig {
    pub image: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl Validate for TextureConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        if self.image.trim().is_empty() {
            return Err("texture image path must not be empty".to_string());
        }
        check::finite("texture horizontal offset", self.offset.x)?;
        check::finite("texture vertical offset", self.offset.y)?;
        check::finite("texture layer depth", self.z)?;
        Ok(())
    }
}

impl TextureConfig {
    pub fn into_feature(self, assets: &AssetServer) -> Texture {
        Texture {
            image: assets.load(format!("texture/{}", self.image)),
            offset: self.offset,
            z: self.z,
            animated: self.animated,
        }
    }
}
