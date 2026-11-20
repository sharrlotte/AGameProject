use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::validate::{Validate, check};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MissionDef {
    pub name: String,
    pub briefing: String,
    #[serde(default)]
    pub ai_level: Option<u8>,
    pub spawn: Vec2,
    pub enemies: Vec<ShipSpawn>,
    pub objective: MissionObjective,
}

#[derive(Deserialize, Clone, PartialEq, Debug, Serialize)]
pub struct ShipSpawn {
    pub team: u8,
    pub position: Vec2,
    pub schematic_name: String,
    #[serde(default)]
    pub ai_level: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MissionObjective {
    DestroyEnemyCore,
}

impl Validate for MissionDef {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("mission name must not be empty".to_string());
        }
        if let Some(level) = self.ai_level {
            check::bounded_int("mission AI difficulty level", level, 1, 10)?;
        }
        for enemy in &self.enemies {
            if enemy.schematic_name.trim().is_empty() {
                return Err("enemy schematic name must not be empty".to_string());
            }
            if let Some(level) = enemy.ai_level {
                check::bounded_int("enemy AI difficulty level", level, 1, 10)?;
            }
        }
        Ok(())
    }
}
