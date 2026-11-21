use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::validate::{Validate, check};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct SkirmishDef {
    #[serde(default)]
    pub ai_level: Option<u8>,
    pub spawn: Vec2,
    pub enemies: Vec<ShipSpawn>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct ShipSpawn {
    pub team: u8,
    pub position: Vec2,
    pub schematic_name: String,
    #[serde(default)]
    pub ai_level: Option<u8>,
}

impl Validate for SkirmishDef {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        if let Some(level) = self.ai_level {
            check::bounded_int("skirmish AI level", level, 1, 10)?;
        }
        for enemy in &self.enemies {
            if enemy.schematic_name.trim().is_empty() {
                return Err("enemy schematic name must not be empty".to_string());
            }
        }
        Ok(())
    }
}
