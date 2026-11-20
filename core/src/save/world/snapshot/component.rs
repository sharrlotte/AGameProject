use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::block::BlockRegistry;

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
pub enum PatternSave {
    Single { state: SingleShootPatternState },
    Burst { state: BurstShootPatternState },
    Laser { shooting: LaserBeamPatternShootingState },
    Pulling,
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
    pub pattern: Option<PatternSave>,
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

impl BlockSnapshot {
    fn validate_blocks(blocks: &[BlockSnapshot], registry: &BlockRegistry, require_core: bool) ->Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut ids = HashSet::new();
    
        for block in blocks {
            if !ids.insert(block.placement.id) {
                errors.push(format!("duplicate block id {}", block.placement.id));
            }
            if registry.get_symbol(&block.placement.block_type).is_none() {
                errors.push(format!("unknown block type {:?}", block.placement.block_type));
            }
            errors.extend(block.validate().err().unwrap_or_default());
        }
    
        if require_core {
            let placements = blocks.iter().map(|b| b.placement.clone()).collect::<Vec<_>>();
            errors.extend(registry.validate_ship(&placements).err().unwrap_or_default());
        }
    
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.placement.id.is_nil() {
            errors.push("block has a nil id".to_string());
        }
        if !self.max_health.is_finite() || self.max_health <= 0.0 {
            errors.push(format!("block {} has invalid max health {}", self.placement.id, self.max_health));
        }
        if !self.health.is_finite() || self.health < 0.0 || self.health > self.max_health {
            errors.push(format!("block {} has invalid health {}", self.placement.id, self.health));
        }
        if let Some(charge) = self.battery_charge
            && (!charge.is_finite() || charge < 0.0)
        {
            errors.push(format!("block {} has invalid battery charge {charge}", self.placement.id));
        }
        if let Some(turret) = &self.turret
            && let Some(charge) = turret.charge
            && (!charge.is_finite() || charge < 0.0)
        {
            errors.push(format!("block {} has invalid turret charge {charge}", self.placement.id));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

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
                active: s.active(),
                shoot_at: s.target().map(|_| s.aim_point()),
                barrel_rotation,
                charge,
            }
        });
        let ammo = world.get::<AmmoState>(entity).map(|s| s.current());
        let pattern = if let Some(state) = world.get::<SingleShootPatternState>(entity) {
            Some(PatternSave::Single { state: state.clone() })
        } else if let Some(state) = world.get::<BurstShootPatternState>(entity) {
            Some(PatternSave::Burst { state: state.clone() })
        } else if let Some(state) = world.get::<LaserBeamPatternState>(entity) {
            Some(PatternSave::Laser { shooting: state.shooting.clone() })
        } else if world.get::<PullingBeamPatternState>(entity).is_some() {
            Some(PatternSave::Pulling)
        } else {
            None
        };

        Self {
            placement,
            health: health.health,
            max_health: health.max_health,
            battery_charge,
            turret,
            ammo,
            pattern,
        }
    }

    pub fn apply(self, commands: &mut Commands, block: Entity) {
        let max_health = if self.max_health.is_finite() && self.max_health > 0.0 {
            self.max_health
        } else {
            warn!("block {} has invalid max health {}; defaulting to 1", self.placement.id, self.max_health);
            1.0
        };
        let health = if self.health.is_finite() && self.health >= 0.0 {
            self.health.min(max_health)
        } else {
            warn!("block {} has invalid health {}; defaulting to max", self.placement.id, self.health);
            max_health
        };
        commands.entity(block).insert(Health { health, max_health });

        if let Some(charge) = self.battery_charge {
            if charge.is_finite() && charge >= 0.0 {
                commands.entity(block).insert(Charge { value: charge });
            } else {
                warn!("block {} has invalid battery charge {charge}; skipping", self.placement.id);
            }
        }

        if let Some(turret) = self.turret {
            commands.entity(block).insert(
                if turret.active {
                    TurretState::Ready { aim_point: turret.shoot_at.unwrap_or_default(), target: None }
                } else {
                    TurretState::Idle
                },
            );
            if let Some(charge) = turret.charge {
                if charge.is_finite() && charge >= 0.0 {
                    commands.entity(block).insert(TurretCharge { value: charge });
                } else {
                    warn!("block {} has invalid turret charge {charge}; skipping", self.placement.id);
                }
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
        match self.pattern {
            Some(PatternSave::Single { state }) => {
                commands.entity(block).insert(state);
            }
            Some(PatternSave::Burst { state }) => {
                commands.entity(block).insert(state);
            }
            Some(PatternSave::Laser { shooting }) => {
                commands.entity(block).insert(LaserBeamPatternState {
                    beam: None,
                    shooting,
                });
            }
            Some(PatternSave::Pulling) => {
                commands.entity(block).insert(PullingBeamPatternState { beam: None });
            }
            None => {}
        }
    }
}

impl ShipSnapshot {
    pub fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
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
        errors.extend(BlockSnapshot::validate_blocks(&self.blocks, registry, true).err().unwrap_or_default());
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl DebrisSnapshot {
    pub fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.team >= 16 {
            errors.push(format!("debris has invalid team {}", self.team));
        }
        if !self.remaining_secs.is_finite() || self.remaining_secs < 0.0 {
            errors.push(format!("debris has invalid lifetime {}", self.remaining_secs));
        }
        errors.extend(BlockSnapshot::validate_blocks(&self.blocks, registry, false).err().unwrap_or_default());
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::block::{BlockPlacement, Rot90};

    fn block(health: f32, max_health: f32) -> BlockSnapshot {
        BlockSnapshot {
            placement: BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            health,
            max_health,
            battery_charge: None,
            turret: None,
            ammo: None,
            pattern: None,
        }
    }

    #[test]
    fn invalid_health_is_rejected() {
        assert!(block(20.0, 10.0).validate().is_err());
        assert!(block(-1.0, 10.0).validate().is_err());
        assert!(block(10.0, 0.0).validate().is_err());
        assert!(block(5.0, 10.0).validate().is_ok());
    }

    #[test]
    fn one_pattern_per_block_is_representable_only_once() {
        let mut block = block(10.0, 10.0);
        block.pattern = Some(PatternSave::Pulling);
        assert!(block.validate().is_ok());
    }
}
