use std::collections::HashMap;
use std::num::NonZeroU32;

use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::ammo::{Ammo, AmmoBuilder};
use crate::validate::NonEmpty;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AmmoConfig {
    pub capacity: NonZeroU32,
    pub crafting: HashMap<NonEmpty<String>, NonZeroU32>,
    pub icon: Option<String>,
}

impl AmmoConfig {
    pub fn into_ammo(self, assets: &AssetServer) -> Ammo {
        AmmoBuilder::build(AmmoBuilder {
            capacity: self.capacity.get(),
            crafting: self
                .crafting
                .into_iter()
                .map(|(k, v)| (k.into_inner(), v.get()))
                .collect(),
            icon: self.icon.map(|icon| assets.load(format!("texture/{icon}"))),
        })
    }
}
