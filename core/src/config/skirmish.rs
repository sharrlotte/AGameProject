use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::validate::{NonEmpty, SkillLevel};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct SkirmishDef {
    #[serde(default)]
    pub ai_level: Option<SkillLevel>,
    pub spawn: Vec2,
    pub enemies: Vec<ShipSpawn>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShipSpawn {
    pub team: u8,
    pub position: Vec2,
    pub schematic_name: NonEmpty<String>,
    #[serde(default)]
    pub ai_level: Option<SkillLevel>,
}
