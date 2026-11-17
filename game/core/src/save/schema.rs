use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    BlockPlacement, BurstShootPatternState, LaserBeamPatternShootingState, SingleShootPatternState,
};
use crate::content::ship::ai::AiProfile;

/// Version of the self-contained world slot format. Old v1 world data and
/// per-ship files are deliberately not read.
pub const SAVE_SCHEMA_VERSION: u32 = 2;
pub const SAVE_SLOTS: usize = 5;

/// Version of the standalone ship design format.
pub const DESIGN_SCHEMA_VERSION: u32 = 1;

/// Turret runtime state. `target` is intentionally omitted: targets are
/// reacquired after load instead of restoring stale entity ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TurretSaveState {
    pub active: bool,
    pub shoot_at: Option<Vec2>,
}

/// Everything needed to recreate one block's mutable runtime state.
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

/// 2D transform snapshot (Rapier games do not need the 3rd axis).
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

/// AI runtime state. The concrete behavior rule set is expanded in Phase 6;
/// this captures the selected behavior and its think timer.
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

/// One self-contained save slot. All ships are embedded; no per-ship files are
/// read or written anywhere in the save pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSave {
    pub version: u32,
    pub slot: u8,
    pub saved_at_unix_ms: u64,
    pub battle: BattleSave,
    pub camera: CameraSave,
    pub ships: Vec<ShipSnapshot>,
    pub drops: Vec<DropSave>,
    pub rng: RngSave,
}

crate::ron_document!(WorldSave, SAVE_SCHEMA_VERSION, "world save");

/// Lightweight slot listing for the save/load menus. Read from the same
/// `WorldSave` document so metadata can never diverge from the real snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SlotInfo {
    pub slot: u8,
    pub occupied: bool,
    pub saved_at_unix_ms: u64,
    pub mission: String,
    pub ship_count: usize,
    pub elapsed_secs: f32,
}
