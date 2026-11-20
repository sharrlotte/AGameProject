use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::power::Charge;
use crate::content::block::feature::turret::{TurretBarrel, TurretCharge, TurretState};
use crate::content::block::{
    BlockPlacement, BurstShootPatternState, GridBlock, Health, LaserBeamPatternShootingState,
    LaserBeamPatternState, PullingBeamPatternState, SingleShootPatternState,
};
use crate::content::ship::ai::{Doctrine, Posture, SupportKind};

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
    pub items: std::collections::BTreeMap<String, u32>,
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

impl BlockSnapshot {
    pub fn capture(world: &World, grid_block: &GridBlock) -> Self {
        let entity = grid_block.entity;
        let placement = grid_block.placement.clone();
        let health = world.get::<Health>(entity).cloned().unwrap_or_default();
        let battery_charge = world.get::<Charge>(entity).map(|c| c.value);
        let turret = world.get::<TurretState>(entity).map(|s| {
            let barrel_rotation = world
                .get::<TurretBarrel>(entity)
                .and_then(|b| world.get::<Transform>(b.0))
                .map(|t| t.rotation.to_euler(EulerRot::ZYX).0)
                .unwrap_or(0.0);

            let charge = world.get::<TurretCharge>(entity).map(|c| c.value);

            TurretSaveState {
                active: s.active,
                shoot_at: s.shoot_at,
                barrel_rotation,
                charge,
            }
        });
        let ammo = world.get::<AmmoState>(entity).map(|s| s.current());
        let single_pattern = world.get::<SingleShootPatternState>(entity).cloned();
        let burst_pattern = world.get::<BurstShootPatternState>(entity).cloned();
        let laser_pattern = world
            .get::<LaserBeamPatternState>(entity)
            .map(|s| s.shooting.clone());
        let pulling_pattern = world.get::<PullingBeamPatternState>(entity).is_some();

        Self {
            placement,
            health: health.health,
            max_health: health.max_health,
            battery_charge,
            turret,
            ammo,
            single_pattern,
            burst_pattern,
            laser_pattern,
            pulling_pattern,
        }
    }

    pub fn apply(self, commands: &mut Commands, block: Entity) {
        commands.entity(block).insert(Health {
            health: self.health,
            max_health: self.max_health,
        });

        if let Some(charge) = self.battery_charge {
            commands.entity(block).insert(Charge { value: charge });
        }

        if let Some(turret) = self.turret {
            commands.entity(block).insert(TurretState {
                active: turret.active,
                target: None,
                aim_point: turret.shoot_at.unwrap_or_default(),
                can_fire: false,
                shoot_at: turret.shoot_at,
            });
            if let Some(charge) = turret.charge {
                commands
                    .entity(block)
                    .insert(TurretCharge { value: charge });
            }
            if turret.barrel_rotation != 0.0 {
                let rot = turret.barrel_rotation;
                commands.entity(block).queue(move |mut entity: EntityWorldMut| {
                    if let Some(&TurretBarrel(barrel)) = entity.get::<TurretBarrel>() {
                        entity.world_scope(move |world| {
                            if let Some(mut transform) = world.get_mut::<Transform>(barrel) {
                                transform.rotation = Quat::from_rotation_z(rot);
                            }
                        });
                    }
                });
            }
        }
        if let Some(ammo) = self.ammo {
            commands.entity(block).insert(AmmoState::new(ammo));
        }
        if let Some(state) = self.single_pattern {
            commands.entity(block).insert(state);
        }
        if let Some(state) = self.burst_pattern {
            commands.entity(block).insert(state);
        }
        if let Some(shooting) = self.laser_pattern {
            commands.entity(block).insert(LaserBeamPatternState {
                targets: Vec::new(),
                beam: None,
                shooting,
            });
        }
        if self.pulling_pattern {
            commands
                .entity(block)
                .insert(PullingBeamPatternState { beam: None });
        }
    }
}
