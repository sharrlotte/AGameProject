use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MissionDef {
    pub name: String,
    pub briefing: String,
    pub player: ShipSpawn,
    pub enemies: Vec<ShipSpawn>,
    pub objective: MissionObjective,
}

#[derive(Deserialize, Clone, PartialEq, Debug, Serialize)]
pub struct ShipSpawn {
    pub team: u8,
    pub position: Vec2,
    pub schematic_name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MissionObjective {
    DestroyEnemyCore,
    // TODO: Survive { seconds: f32 }, Collect { steel: u32 }, Escort { ... } — future missions
}
