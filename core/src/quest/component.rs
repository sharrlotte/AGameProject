use std::collections::HashMap;
use std::num::NonZeroU32;

use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::quest::def::validate_catalog;
use crate::validate::{NonEmpty, OrderedRange, PositiveF32, non_zero_u32};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum QuestId {
    Authored(String),
    Chunk(IVec2),
}

impl QuestId {
    pub fn key(&self) -> String {
        match self {
            Self::Authored(id) => format!("authored:{id}"),
            Self::Chunk(chunk) => format!("chunk:{}:{}", chunk.x, chunk.y),
        }
    }

    pub fn is_generated(&self) -> bool {
        matches!(self, Self::Chunk(_))
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum QuestStatus {
    #[default]
    Offered,
    Active,
    Completed,
}

impl QuestStatus {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Offered => 0,
            Self::Active => 1,
            Self::Completed => 2,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Offered),
            1 => Some(Self::Active),
            2 => Some(Self::Completed),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum QuestAcceptance {
    #[default]
    Auto,
    Manual,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum QuestPrereqMode {
    #[default]
    Completed,
    Started,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestOffer {
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub enum QuestObjectiveDef {
    DestroyEnemies { id: String, count: u32 },
    ReachPosition { id: String, position: Vec2, radius: f32 },
    CollectItem { id: String, item: String, count: u32 },
}

impl QuestObjectiveDef {
    pub fn id(&self) -> &str {
        match self {
            Self::DestroyEnemies { id, .. }
            | Self::ReachPosition { id, .. }
            | Self::CollectItem { id, .. } => id,
        }
    }

    pub fn target(&self) -> u32 {
        match self {
            Self::DestroyEnemies { count, .. } | Self::CollectItem { count, .. } => *count,
            Self::ReachPosition { .. } => 1,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::DestroyEnemies { .. } => "Destroy Enemies".to_string(),
            Self::ReachPosition { .. } => "Reach Location".to_string(),
            Self::CollectItem { item, .. } => item.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct QuestDef {
    pub id: QuestId,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub requires: Vec<QuestId>,
    #[serde(default)]
    pub requires_mode: QuestPrereqMode,
    #[serde(default)]
    pub acceptance: QuestAcceptance,
    #[serde(default)]
    pub offer: Option<QuestOffer>,
    pub objectives: Vec<QuestObjectiveDef>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(try_from = "Vec<QuestDef>")]
pub struct ValidCatalog(Vec<QuestDef>);

impl ValidCatalog {
    pub fn iter(&self) -> std::slice::Iter<'_, QuestDef> {
        self.0.iter()
    }
}

impl TryFrom<Vec<QuestDef>> for ValidCatalog {
    type Error = String;

    fn try_from(quests: Vec<QuestDef>) -> Result<Self, Self::Error> {
        validate_catalog(&quests)?;
        Ok(Self(quests))
    }
}

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct QuestCatalogDef {
    pub quests: ValidCatalog,
}

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct QuestGenConfig {
    pub max_active_generated: usize,
    pub notification_radius: PositiveF32,
    pub manual_acceptance_every: NonZeroU32,
    pub chunk_size: PositiveF32,
    pub destroy_count: OrderedRange<NonZeroU32>,
    pub collect_count: OrderedRange<NonZeroU32>,
    pub reach_radius: OrderedRange<PositiveF32>,
    pub item_pool: NonEmpty<Vec<NonEmpty<String>>>,
}

impl Default for QuestGenConfig {
    fn default() -> Self {
        Self {
            max_active_generated: 2,
            notification_radius: PositiveF32::from_const(2500.0),
            manual_acceptance_every: non_zero_u32(2),
            chunk_size: PositiveF32::from_const(50_000.0),
            destroy_count: OrderedRange::<NonZeroU32>::from_const(2, 6),
            collect_count: OrderedRange::<NonZeroU32>::from_const(3, 10),
            reach_radius: OrderedRange::<PositiveF32>::from_const(500.0, 2000.0),
            item_pool: NonEmpty::single(NonEmpty::from_static_str("steel")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestProgress {
    pub quest_id: QuestId,
    pub status: QuestStatus,
    pub objective_counts: HashMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestRecord {
    pub def: QuestDef,
    pub progress: QuestProgress,
}

impl QuestRecord {
    pub fn objective_progress_mut(&mut self, id: &str) -> &mut u32 {
        self.progress.objective_counts.entry(id.to_string()).or_insert(0)
    }

    pub fn objective_progress(&self, id: &str) -> u32 {
        self.progress.objective_counts.get(id).copied().unwrap_or(0)
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct QuestCatalogValue(pub HashMap<QuestId, QuestDef>);

#[derive(Resource, Default, Debug, Clone)]
pub struct QuestGenConfigValue(pub QuestGenConfig);

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct QuestCompleted {
    pub quest_id: QuestId,
}

#[derive(Resource)]
pub struct QuestHandles {
    pub catalog: Handle<QuestCatalogDef>,
    pub generation: Handle<QuestGenConfig>,
}

#[derive(Resource, Default)]
pub struct QuestAssetsReady(pub bool);

#[derive(Resource, Default, Debug)]
pub struct QuestTracker {
    pub records: HashMap<QuestId, QuestRecord>,
}

impl QuestTracker {
    pub fn status(&self, id: &QuestId) -> Option<QuestStatus> {
        self.records.get(id).map(|record| record.progress.status)
    }

    pub fn contains(&self, id: &QuestId) -> bool {
        self.records.contains_key(id)
    }

    pub fn active_ids(&self) -> Vec<QuestId> {
        self.records
            .iter()
            .filter(|(_, record)| record.progress.status == QuestStatus::Active)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn count_generated_active(&self) -> usize {
        self.records
            .iter()
            .filter(|(id, record)| {
                id.is_generated() && record.progress.status == QuestStatus::Active
            })
            .count()
    }

    pub fn insert(&mut self, record: QuestRecord) {
        self.records.insert(record.progress.quest_id.clone(), record);
    }
}

#[derive(Resource, Default, Debug)]
pub struct QuestOfferQueue {
    pub pending: Vec<QuestId>,
    pub open: Option<QuestId>,
}

#[derive(Resource, Default, Debug)]
pub struct VisibleOffers(pub Vec<QuestId>);

#[derive(Component, Clone, Debug)]
pub struct OpenOffer(pub QuestId);

#[derive(Component, Clone, Debug)]
pub struct AcceptOffer(pub QuestId);

#[derive(Component, Clone, Debug)]
pub struct CloseOffer;

#[derive(Component)]
pub struct QuestTrackerText;

#[derive(Component)]
pub struct QuestOfferList;

#[derive(Component)]
pub struct QuestOfferDialogHost;
