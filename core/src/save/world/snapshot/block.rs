use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::block::BlockRegistry;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::power::Charge;
use crate::content::block::feature::turret::{TurretBarrel, TurretCharge, TurretState};
use crate::content::block::{
    BlockPlacement, BurstShootPatternState, GridBlock, Health, LaserBeamPatternShootingState,
    LaserBeamPatternState, PullingBeamPatternState, SingleShootPatternState,
};
use super::sanitize::{sanitize_charge, sanitize_health};

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

impl BlockSnapshot {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.placement.id.is_nil() {
            errors.push("block has a nil id".to_string());
        }
        if !self.max_health.is_finite() || self.max_health <= 0.0 {
            errors.push(format!(
                "block {} has invalid max health {}",
                self.placement.id, self.max_health
            ));
        }
        if !self.health.is_finite() || self.health < 0.0 || self.health > self.max_health {
            errors.push(format!(
                "block {} has invalid health {}",
                self.placement.id, self.health
            ));
        }
        if let Some(charge) = self.battery_charge
            && (!charge.is_finite() || charge < 0.0)
        {
            errors.push(format!(
                "block {} has invalid battery charge {charge}",
                self.placement.id
            ));
        }
        if let Some(turret) = &self.turret
            && let Some(charge) = turret.charge
            && (!charge.is_finite() || charge < 0.0)
        {
            errors.push(format!(
                "block {} has invalid turret charge {charge}",
                self.placement.id
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl BlockSnapshot {
    pub fn validate_blocks(
        blocks: &[BlockSnapshot],
        registry: &BlockRegistry,
        require_core: bool,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut ids = HashSet::new();

        for block in blocks {
            if !ids.insert(block.placement.id) {
                errors.push(format!("duplicate block id {}", block.placement.id));
            }
            if registry.get_symbol(&block.placement.block_type).is_none() {
                errors.push(format!(
                    "unknown block type {:?}",
                    block.placement.block_type
                ));
            }
            errors.extend(block.validate().err().unwrap_or_default());
        }

        if require_core {
            let placements = blocks.iter().map(|b| b.placement.clone()).collect::<Vec<_>>();
            errors.extend(registry.validate_ship(&placements).err().unwrap_or_default());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
            Some(PatternSave::Single {
                state: state.clone(),
            })
        } else if let Some(state) = world.get::<BurstShootPatternState>(entity) {
            Some(PatternSave::Burst {
                state: state.clone(),
            })
        } else if let Some(state) = world.get::<LaserBeamPatternState>(entity) {
            Some(PatternSave::Laser {
                shooting: state.shooting.clone(),
            })
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
        let (health, max_health) = sanitize_health(self.placement.id, self.health, self.max_health);
        commands.entity(block).insert(Health { health, max_health });

        if let Some(charge) = sanitize_charge(self.placement.id, "battery", self.battery_charge) {
            commands.entity(block).insert(Charge { value: charge });
        }

        if let Some(turret) = self.turret {
            commands.entity(block).insert(if turret.active {
                TurretState::Ready {
                    aim_point: turret.shoot_at.unwrap_or_default(),
                    target: None,
                }
            } else {
                TurretState::Idle
            });
            if let Some(charge) = sanitize_charge(self.placement.id, "turret", turret.charge) {
                commands.entity(block).insert(TurretCharge { value: charge });
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
