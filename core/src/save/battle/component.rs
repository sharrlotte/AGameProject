use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    BlockPlacement, BurstShootPatternState, LaserBeamPatternShootingState, SingleShootPatternState,
};
use crate::content::ship::Ship;
use crate::content::ship::ai::{AiBehavior, AiProfile};

pub const SAVE_SCHEMA_VERSION: u32 = 5;

#[derive(Message)]
pub struct SaveWorldEvent;

#[derive(Message)]
pub struct LoadWorldEvent;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingExitAction {
    #[default]
    None,
    Home,
}

#[derive(Resource, Default, Debug)]
pub struct SaveResult {
    pub message: Option<String>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleSaveState {
    pub has_save: bool,
}

pub struct CaptureContext<'a> {
    pub world: &'a World,
    pub mission: &'a str,
    pub elapsed_secs: f32,
    pub outcome: Option<String>,
    pub saved_at_unix_ms: u64,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TurretSaveState {
    pub active: bool,
    pub shoot_at: Option<Vec2>,
    #[serde(default)]
    pub barrel_rotation: f32,
    #[serde(default)]
    pub charge: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSnapshot {
    pub placement: BlockPlacement,
    pub health: f32,
    pub max_health: f32,
    #[serde(default)]
    pub battery_charge: Option<f32>,
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
pub struct CargoSave {
    pub used: u32,
    pub items: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AiBehaviorSave {
    #[default]
    Idle,
    Wander {
        point: Vec2,
    },
    Patrol {
        around: Vec2,
        radius: f32,
    },
    Guard {
        target: Option<Uuid>,
        radius: f32,
    },
    Follow {
        target: Option<Uuid>,
        radius: f32,
    },
    Regroup {
        position: Vec2,
        radius: f32,
    },
    Kite {
        target: Option<Uuid>,
        range: f32,
    },
    Chase {
        target: Option<Uuid>,
    },
    Attack {
        target: Option<Uuid>,
    },
    Flee {
        from: Option<Uuid>,
        distance: f32,
    },
}

impl AiBehaviorSave {
    pub fn from_runtime(world: &World, behavior: &AiBehavior) -> Self {
        let ship_uuid = |target: Option<Entity>| -> Option<Uuid> {
            target.and_then(|e| world.get::<Ship>(e).map(|s| s.id))
        };

        match *behavior {
            AiBehavior::Idle => Self::Idle,
            AiBehavior::Wander { point } => Self::Wander { point },
            AiBehavior::Patrol { around, radius } => Self::Patrol { around, radius },
            AiBehavior::Guard { target, radius } => Self::Guard {
                target: ship_uuid(target),
                radius,
            },
            AiBehavior::Follow { target, radius } => Self::Follow {
                target: ship_uuid(target),
                radius,
            },
            AiBehavior::Regroup { position, radius } => Self::Regroup { position, radius },
            AiBehavior::Kite { target, range } => Self::Kite {
                target: ship_uuid(target),
                range,
            },
            AiBehavior::Chase { target } => Self::Chase {
                target: ship_uuid(target),
            },
            AiBehavior::Attack { target } => Self::Attack {
                target: ship_uuid(target),
            },
            AiBehavior::Flee { from, distance } => Self::Flee {
                from: ship_uuid(from),
                distance,
            },
        }
    }

    pub fn into_runtime(self, ship_map: &HashMap<Uuid, Entity>) -> AiBehavior {
        let resolve = |id: Option<Uuid>| -> Option<Entity> {
            id.and_then(|uuid| ship_map.get(&uuid).copied())
        };

        match self {
            Self::Idle => AiBehavior::Idle,
            Self::Wander { point } => AiBehavior::Wander { point },
            Self::Patrol { around, radius } => AiBehavior::Patrol { around, radius },
            Self::Guard { target, radius } => AiBehavior::Guard {
                target: resolve(target),
                radius,
            },
            Self::Follow { target, radius } => AiBehavior::Follow {
                target: resolve(target),
                radius,
            },
            Self::Regroup { position, radius } => AiBehavior::Regroup { position, radius },
            Self::Kite { target, range } => AiBehavior::Kite {
                target: resolve(target),
                range,
            },
            Self::Chase { target } => AiBehavior::Chase {
                target: resolve(target),
            },
            Self::Attack { target } => AiBehavior::Attack {
                target: resolve(target),
            },
            Self::Flee { from, distance } => AiBehavior::Flee {
                from: resolve(from),
                distance,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AiRuntimeSave {
    pub current: AiBehaviorSave,
    pub spawn_position: Vec2,
    pub rule_cooldowns: Vec<f32>,
    pub timer_secs: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipSnapshot {
    pub id: Uuid,
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    #[serde(default)]
    pub cargo: CargoSave,
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
    #[serde(default)]
    pub control: ControlSave,
    pub ships: Vec<ShipSnapshot>,
    pub drops: Vec<DropSave>,
    pub rng: RngSave,
}

crate::ron_document!(WorldSave, SAVE_SCHEMA_VERSION, "world save");
