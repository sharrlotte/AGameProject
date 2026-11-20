use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::ammo::{Ammo, AmmoBuilder};
use crate::validate::{Validate, check};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AmmoConfig {
    pub capacity: u32,
    pub crafting: HashMap<String, u32>,
    pub icon: Option<String>,
}

impl Validate for AmmoConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::at_least("ammo capacity", self.capacity, 1)?;
        for (item, &qty) in &self.crafting {
            if item.trim().is_empty() {
                return Err("ammo crafting item name cannot be empty".to_string());
            }
            if qty == 0 {
                return Err(format!("ammo crafting quantity for {item} must be at least 1"));
            }
        }
        Ok(())
    }
}

impl AmmoConfig {
    pub fn into_ammo(self, assets: &AssetServer) -> Ammo {
        AmmoBuilder::build(AmmoBuilder {
            capacity: self.capacity,
            crafting: self.crafting,
            icon: self.icon.map(|icon| assets.load(format!("texture/{icon}"))),
        })
    }
}
