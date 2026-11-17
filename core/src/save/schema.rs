use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    BlockPlacement, BurstShootPatternState, LaserBeamPatternShootingState, SingleShootPatternState,
};
use crate::content::ship::ai::AiProfile;

pub const SAVE_SCHEMA_VERSION: u32 = 3;

pub const CAMPAIGN_SCHEMA_VERSION: u32 = 1;

pub const DESIGN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TurretSaveState {
    pub active: bool,
    pub shoot_at: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSnapshot {
    pub placement: BlockPlacement,
    pub health: f32,
    pub max_health: f32,
    pub turret: Option<TurretSaveState>,
    pub ammo: Option<u32>,
    pub single_pattern: Option<SingleShootPatternState>,
    pub burst_pattern: Option<BurstShootPatternState>,
    pub laser_pattern: Option<LaserBeamPatternShootingState>,
    pub pulling_pattern: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TransformSave {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VelocitySave {
    pub x: f32,
    pub y: f32,
    pub angvel: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AiRuntimeSave {
    pub behavior: String,
    pub timer_secs: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipSnapshot {
    pub id: Uuid,
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    pub ai: AiProfile,
    pub ai_runtime: AiRuntimeSave,
    pub blocks: Vec<BlockSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BattleSave {
    pub mission: String,
    pub elapsed_secs: f32,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CameraSave {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropSave {
    pub position: Vec2,
    pub item_type: String,
    pub amount: u32,
    pub remaining_secs: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RngSave {
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSave {
    pub version: u32,
    pub saved_at_unix_ms: u64,
    pub battle: BattleSave,
    pub camera: CameraSave,
    pub ships: Vec<ShipSnapshot>,
    pub drops: Vec<DropSave>,
    pub rng: RngSave,
}

crate::ron_document!(WorldSave, SAVE_SCHEMA_VERSION, "world save");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CampaignSave {
    pub version: u32,
    pub current_mission: String,
    pub completed_missions: Vec<String>,
}

impl Default for CampaignSave {
    fn default() -> Self {
        Self {
            version: CAMPAIGN_SCHEMA_VERSION,
            current_mission: "quick-battle".to_string(),
            completed_missions: Vec::new(),
        }
    }
}

crate::ron_document!(CampaignSave, CAMPAIGN_SCHEMA_VERSION, "campaign save");
