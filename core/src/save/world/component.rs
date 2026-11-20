use std::fmt;
use std::io;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::save::world::store::WorldStore;

pub const WORLD_DB_SCHEMA_VERSION: u32 = 1;

#[derive(Message)]
pub struct SaveWorldEvent;

#[derive(Message)]
pub struct LoadSlotEvent {
    pub slot: SlotSummary,
}

#[derive(Message)]
pub struct CreateSlotEvent {
    pub name: String,
}

#[derive(Message)]
pub struct StartQuickBattleEvent;

#[derive(Resource, Clone, Debug, PartialEq, Eq, Default)]
pub enum WorldSession {
    SaveSlot { slot_id: Uuid, path: PathBuf },
    #[default]
    QuickBattle,
}

#[derive(Resource, Default)]
pub struct ActiveWorldStore(pub Option<Box<dyn WorldStore>>);

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorReturnTarget {
    #[default]
    Home,
    Game,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingExitAction {
    #[default]
    None,
    Home,
}

#[derive(Resource)]
pub struct AutoSaveTimer(pub Timer);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotSummary {
    pub slot_id: Uuid,
    pub name: String,
    pub seed: u64,
    pub play_time_secs: f32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMeta {
    pub name: String,
    pub seed: u64,
    pub play_time_secs: f32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub camera_blob: Vec<u8>,
    pub control_blob: Vec<u8>,
    pub rng_blob: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ActiveStateSnapshot {
    pub bodies_blob: Vec<u8>,
    pub drops_blob: Vec<u8>,
    pub intel_blob: Vec<u8>,
}

#[derive(Debug)]
pub enum WorldStoreError {
    Io(String),
    Sql(String),
    Encode(String),
    Decode(String),
    VersionMismatch { expected: u32, found: u32 },
    NotFound(String),
    InvalidData(String),
}

impl fmt::Display for WorldStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Sql(e) => write!(f, "sqlite error: {e}"),
            Self::Encode(e) => write!(f, "encode error: {e}"),
            Self::Decode(e) => write!(f, "decode error: {e}"),
            Self::VersionMismatch { expected, found } => write!(
                f,
                "incompatible world schema: expected version {expected}, found {found}"
            ),
            Self::NotFound(path) => write!(f, "world store not found: {path}"),
            Self::InvalidData(e) => write!(f, "invalid world data: {e}"),
        }
    }
}

impl std::error::Error for WorldStoreError {}

impl From<io::Error> for WorldStoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
