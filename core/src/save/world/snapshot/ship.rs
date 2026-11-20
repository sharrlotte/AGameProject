use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::BlockRegistry;
use crate::content::ship::ai::{Doctrine, Posture, SupportKind};
use crate::validate::Validate;
use super::block::BlockSnapshot;

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
pub struct CargoSave {
    pub used: u32,
    pub items: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TacticSave {
    Engage {
        target: Option<Uuid>,
        range: f32,
        posture: Posture,
    },
    Recharge {
        threat: Option<Uuid>,
        safe_range: f32,
    },
    Retreat {
        threat: Option<Uuid>,
    },
    Support {
        ally: Option<Uuid>,
        kind: SupportKind,
    },
    Regroup {
        position: Vec2,
    },
    Investigate {
        position: Vec2,
    },
    Search {
        waypoint: Vec2,
    },
}

impl Default for TacticSave {
    fn default() -> Self {
        Self::Search {
            waypoint: Vec2::ZERO,
        }
    }
}

impl TacticSave {
    pub fn target_uuid(&self) -> Option<Uuid> {
        match self {
            Self::Engage { target, .. }
            | Self::Recharge { threat: target, .. }
            | Self::Retreat { threat: target }
            | Self::Support { ally: target, .. } => *target,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BrainSave {
    pub current: TacticSave,
    pub spawn_position: Vec2,
    pub timer_secs: f32,
    pub strafe_sign: f32,
    pub strafe_timer: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastSeenSave {
    pub target_id: Uuid,
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamAlertSave {
    pub position: Vec2,
    pub ttl: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TeamIntelSave {
    pub team: u8,
    pub last_known: Vec<LastSeenSave>,
    pub alerts: Vec<TeamAlertSave>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipSnapshot {
    pub id: Uuid,
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    #[serde(default)]
    pub cargo: CargoSave,
    pub doctrine: Doctrine,
    pub skill: u8,
    pub brain: BrainSave,
    pub blocks: Vec<BlockSnapshot>,
}

impl Validate for ShipSnapshot {
    type Context = BlockRegistry;
    type Error = Vec<String>;

    fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.team >= 16 {
            errors.push(format!("ship {} has invalid team {}", self.id, self.team));
        }
        if !(1..=10).contains(&self.skill) {
            errors.push(format!("ship {} has invalid skill {}", self.id, self.skill));
        }
        if self.id.is_nil() {
            errors.push("ship has a nil id".to_string());
        }
        if self.blocks.is_empty() {
            errors.push(format!("ship {} has no blocks", self.id));
        }
        errors.extend(
            BlockSnapshot::validate_blocks(&self.blocks, registry, true)
                .err()
                .unwrap_or_default(),
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ShipSnapshot {
    pub fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
        Validate::validate(self, registry)
    }
}
