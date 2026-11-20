use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CameraSave {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    #[serde(default)]
    pub following: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ControlSave {
    pub controlling: Option<Uuid>,
    pub pilot_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropSave {
    pub position: Vec2,
    pub item_type: String,
    pub amount: u32,
    pub remaining_secs: f32,
}
