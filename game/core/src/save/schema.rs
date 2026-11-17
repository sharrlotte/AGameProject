use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    component::{BurstShootPatternState, SingleShootPatternState},
};
use crate::symbol::Symbol;

pub const WORLD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurretSaveState {
    pub active: bool,
    pub shoot_at: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSaveData {
    pub position: IVec2,
    pub block_type: Symbol,
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

/// Read the `version` field from a RON save without deserializing the rest,
/// rejecting unknown versions loudly before any data is trusted.
pub fn read_version(data: &str) -> Result<u32, String> {
    #[derive(Deserialize)]
    struct VersionOnly {
        version: u32,
    }

    let parsed = ron::from_str::<VersionOnly>(data)
        .map_err(|e| format!("incompatible save schema: malformed header ({e})"))?;

    if parsed.version != WORLD_SCHEMA_VERSION {
        return Err(format!(
            "incompatible save schema: expected version {}, found {}",
            WORLD_SCHEMA_VERSION, parsed.version
        ));
    }

    Ok(parsed.version)
}
