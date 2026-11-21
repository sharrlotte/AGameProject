use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::quest::component::QuestStatus;
use crate::save::world::snapshot::{BodySnapshot, CameraSave, ControlSave, DropSave, TeamIntelSave};
use crate::save::world::store::{InMemoryWorldStore, WorldStore};

pub const WORLD_DB_SCHEMA_VERSION: u32 = 3;

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

#[derive(Message)]
pub struct ContinueWorldEvent;

#[derive(Resource)]
pub enum WorldContext {
    QuickBattle { store: Box<dyn WorldStore> },
    SaveSlot { slot_id: Uuid, path: PathBuf, store: Box<dyn WorldStore> },
    Restoring { slot_id: Uuid, path: PathBuf, store: Box<dyn WorldStore> },
}

impl Default for WorldContext {
    fn default() -> Self {
        Self::QuickBattle { store: Box::new(InMemoryWorldStore::default()) }
    }
}

impl fmt::Debug for WorldContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuickBattle { .. } => f.write_str("QuickBattle"),
            Self::SaveSlot { slot_id, path, .. } => {
                f.debug_struct("SaveSlot").field("slot_id", slot_id).field("path", path).finish()
            }
            Self::Restoring { slot_id, path, .. } => {
                f.debug_struct("Restoring").field("slot_id", slot_id).field("path", path).finish()
            }
        }
    }
}

impl WorldContext {
    pub fn store(&self) -> &dyn WorldStore {
        match self {
            Self::QuickBattle { store }
            | Self::SaveSlot { store, .. }
            | Self::Restoring { store, .. } => store.as_ref(),
        }
    }

    pub fn store_mut(&mut self) -> &mut dyn WorldStore {
        match self {
            Self::QuickBattle { store }
            | Self::SaveSlot { store, .. }
            | Self::Restoring { store, .. } => store.as_mut(),
        }
    }

    pub fn slot_id(&self) -> Option<Uuid> {
        match self {
            Self::QuickBattle { .. } => None,
            Self::SaveSlot { slot_id, .. } | Self::Restoring { slot_id, .. } => Some(*slot_id),
        }
    }

    pub fn is_save_slot(&self) -> bool {
        !matches!(self, Self::QuickBattle { .. })
    }

    pub fn is_restoring(&self) -> bool {
        matches!(self, Self::Restoring { .. })
    }

    pub fn begin_restore(&mut self) -> bool {
        match std::mem::replace(self, Self::default()) {
            Self::SaveSlot { slot_id, path, store } => {
                *self = Self::Restoring { slot_id, path, store };
                true
            }
            other => {
                *self = other;
                false
            }
        }
    }

    pub fn take_restoring(&mut self) -> Option<(Uuid, PathBuf, Box<dyn WorldStore>)> {
        match std::mem::replace(self, Self::default()) {
            Self::Restoring { slot_id, path, store } => Some((slot_id, path, store)),
            other => {
                *self = other;
                None
            }
        }
    }
}

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

#[derive(Debug)]
pub struct WorldLoadData {
    pub seed: u64,
    pub play_time_secs: f32,
    pub camera_save: CameraSave,
    pub control_save: ControlSave,
    pub bodies: Vec<BodySnapshot>,
    pub ship_ids: HashSet<Uuid>,
    pub drops: Vec<DropSave>,
    pub intel: Vec<TeamIntelSave>,
    pub quests: Vec<QuestStoreRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuestStoreRow {
    pub quest_id: String,
    pub status: QuestStatus,
    pub record_blob: Vec<u8>,
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
