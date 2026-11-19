use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::{
    BlockPlacement, BurstShootPatternState, LaserBeamPatternShootingState, SingleShootPatternState,
};
use crate::content::ship::Ship;
use crate::content::ship::ai::{Doctrine, Posture, SupportKind, Tactic};

pub const SAVE_SCHEMA_VERSION: u32 = 7;

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
    pub fn from_runtime(world: &World, tactic: &Tactic) -> Self {
        let ship_uuid = |target: Entity| -> Option<Uuid> {
            world.get::<Ship>(target).map(|s| s.id)
        };

        match *tactic {
            Tactic::Engage {
                target,
                range,
                posture,
            } => Self::Engage {
                target: ship_uuid(target),
                range,
                posture,
            },
            Tactic::Recharge { threat, safe_range } => Self::Recharge {
                threat: ship_uuid(threat),
                safe_range,
            },
            Tactic::Retreat { threat } => Self::Retreat {
                threat: ship_uuid(threat),
            },
            Tactic::Support { ally, kind } => Self::Support {
                ally: ship_uuid(ally),
                kind,
            },
            Tactic::Regroup { position } => Self::Regroup { position },
            Tactic::Investigate { position } => Self::Investigate { position },
            Tactic::Search { waypoint } => Self::Search { waypoint },
        }
    }

    pub fn into_runtime(self, ship_map: &HashMap<Uuid, Entity>, fallback_pos: Vec2) -> Tactic {
        let resolve = |id: Option<Uuid>| -> Option<Entity> {
            id.and_then(|uuid| ship_map.get(&uuid).copied())
        };

        match self {
            Self::Engage {
                target,
                range,
                posture,
            } => resolve(target).map_or(Tactic::Search { waypoint: fallback_pos }, |target| {
                Tactic::Engage {
                    target,
                    range,
                    posture,
                }
            }),
            Self::Recharge { threat, safe_range } => {
                resolve(threat).map_or(Tactic::Search { waypoint: fallback_pos }, |threat| {
                    Tactic::Recharge { threat, safe_range }
                })
            }
            Self::Retreat { threat } => {
                resolve(threat).map_or(Tactic::Search { waypoint: fallback_pos }, |threat| {
                    Tactic::Retreat { threat }
                })
            }
            Self::Support { ally, kind } => {
                resolve(ally).map_or(Tactic::Search { waypoint: fallback_pos }, |ally| {
                    Tactic::Support { ally, kind }
                })
            }
            Self::Regroup { position } => Tactic::Regroup { position },
            Self::Investigate { position } => Tactic::Investigate { position },
            Self::Search { waypoint } => Tactic::Search { waypoint },
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
    pub shed_boosters: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebrisSnapshot {
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    pub remaining_secs: f32,
    pub blocks: Vec<BlockSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodySnapshot {
    Ship(ShipSnapshot),
    Debris(DebrisSnapshot),
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
    pub bodies: Vec<BodySnapshot>,
    #[serde(default)]
    pub team_intel: Vec<TeamIntelSave>,
    pub drops: Vec<DropSave>,
    pub rng: RngSave,
}

crate::ron_document!(WorldSave, SAVE_SCHEMA_VERSION, "world save");
