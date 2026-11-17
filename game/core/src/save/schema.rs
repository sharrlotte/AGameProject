use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    component::{BurstShootPatternState, SingleShootPatternState},
};

pub const WORLD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurretSaveState {
    pub active: bool,
    pub shoot_at: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSaveData {
    pub position: IVec2,
    pub block_type: String,
    pub rotation: u8,
    pub health: f32,
    pub max_health: f32,
    pub turret_state: Option<TurretSaveState>,
    pub single_pattern: Option<SingleShootPatternState>,
    pub burst_pattern: Option<BurstShootPatternState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipSave {
    pub version: u32,
    pub id: Uuid,
    pub team: u8,
    pub blocks: Vec<BlockSaveData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSave {
    pub version: u32,
    pub teams: Vec<u8>,
    pub elapsed_secs: f32,
    pub ship_ids: Vec<Uuid>,
}

crate::ron_document!(WorldSave, WORLD_SCHEMA_VERSION, "world save");
crate::ron_document!(ShipSave, WORLD_SCHEMA_VERSION, "ship save");
