//! `CaptureSnapshot`/`RestoreSnapshot` implementations for save data.
//!
//! Runtime ECS state is converted into serializable snapshots here, never in
//! the IO layer. Entity references are never persisted: turret targets, laser
//! beams and AI targets are reacquired after a load.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::constant::LAYER_DROP;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::turret::TurretState;
use crate::content::block::{
    BlockPlacement, BlockRef, BlockRegistry, BurstShootPatternState, DropComponent, Health,
    ItemDrop, LaserBeamPatternState, PullingBeamPatternState, SingleShootPatternState,
    spawn_block_entity,
};
use crate::content::lifetime::Lifetime;
use crate::content::ship::ai::{AiBehavior, AiProfile, AiRuntime};
use crate::content::ship::{Ship, ShipDesign, ShipGrid};
use crate::content::team::Team;
use crate::resource::SceneState;
use crate::save::document::DocumentError;
use crate::save::schema::*;

/// Immutable world state available while capturing a snapshot.
pub struct CaptureContext<'a> {
    pub world: &'a World,
    pub slot: u8,
    pub mission: &'a str,
    pub elapsed_secs: f32,
    pub outcome: Option<String>,
    pub saved_at_unix_ms: u64,
    pub seed: u64,
}

/// Convert runtime state into a serializable snapshot.
pub trait CaptureSnapshot: Sized {
    fn capture(ctx: &CaptureContext, entity: Entity) -> Self;
}

/// Rebuild runtime state from a serializable snapshot. `Commands` is passed
/// explicitly because its two lifetimes cannot be stored in a helper struct
/// without over-constraining callers.
pub trait RestoreSnapshot: Sized {
    fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
        team: Team,
        parent: Option<Entity>,
    ) -> Result<Entity, DocumentError>;
}

fn placement_for(world: &World, block_id: Uuid) -> Option<BlockPlacement> {
    world.iter_entities().find_map(|entity| {
        entity
            .get::<ShipGrid>()
            .and_then(|grid| grid.block(block_id))
            .map(|b| b.placement.clone())
    })
}

impl CaptureSnapshot for BlockSnapshot {
    fn capture(ctx: &CaptureContext, entity: Entity) -> Self {
        let world = ctx.world;
        let placement = world
            .get::<BlockRef>(entity)
            .and_then(|r| placement_for(world, r.id))
            .unwrap_or_else(|| BlockPlacement::new(IVec2::ZERO, "hull", 0));
        let health = world.get::<Health>(entity).cloned().unwrap_or_default();
        let turret = world.get::<TurretState>(entity).map(|s| TurretSaveState {
            active: s.active,
            shoot_at: s.shoot_at,
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
            turret,
            ammo,
            single_pattern,
            burst_pattern,
            laser_pattern,
            pulling_pattern,
        }
    }
}

impl BlockSnapshot {
    /// Apply saved mutable state to a block entity that was just spawned by
    /// `ShipDesign::spawn`. No new entity is created here.
    pub fn apply_to(self, commands: &mut Commands, block: Entity) {
        commands.entity(block).insert(Health {
            health: self.health,
            max_health: self.max_health,
        });

        if let Some(turret) = self.turret {
            commands.entity(block).insert(TurretState {
                active: turret.active,
                target: None,
                aim_point: turret.shoot_at.unwrap_or_default(),
                can_fire: false,
                shoot_at: turret.shoot_at,
            });
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

impl RestoreSnapshot for BlockSnapshot {
    fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
        team: Team,
        parent: Option<Entity>,
    ) -> Result<Entity, DocumentError> {
        let ship = parent.ok_or_else(|| DocumentError::Parse("block has no parent ship".into()))?;
        let def = registry
            .get_symbol(&self.placement.block_type)
            .ok_or_else(|| {
                DocumentError::Parse(format!("unknown block type {}", self.placement.block_type))
            })?;

        let block = spawn_block_entity(commands, &self.placement, def, ship, team, config);
        self.apply_to(commands, block);
        Ok(block)
    }
}

impl CaptureSnapshot for ShipSnapshot {
    fn capture(ctx: &CaptureContext, entity: Entity) -> Self {
        let world = ctx.world;
        let team = world.get::<Team>(entity).map(|t| t.id).unwrap_or(0);
        let transform = world.get::<Transform>(entity).cloned().unwrap_or_default();
        let velocity = world.get::<Velocity>(entity).cloned().unwrap_or_default();
        let ai = world.get::<AiProfile>(entity).cloned().unwrap_or_default();

        let blocks = world
            .get::<ShipGrid>(entity)
            .map(|grid| {
                grid.blocks()
                    .filter(|b| b.entity != Entity::PLACEHOLDER)
                    .map(|b| BlockSnapshot::capture(ctx, b.entity))
                    .collect()
            })
            .unwrap_or_default();

        let behavior = world
            .get::<AiRuntime>(entity)
            .map(|r| r.current.label().to_string())
            .unwrap_or_default();
        let timer_secs = world
            .get::<AiRuntime>(entity)
            .map(|r| r.think_timer.remaining_secs())
            .unwrap_or(0.0);

        Self {
            id: world.get::<Ship>(entity).map(|s| s.id).unwrap_or_default(),
            team,
            transform: TransformSave {
                x: transform.translation.x,
                y: transform.translation.y,
                rotation: transform.rotation.to_euler(EulerRot::ZYX).0,
            },
            velocity: VelocitySave {
                x: velocity.linear.x,
                y: velocity.linear.y,
                angvel: velocity.angular,
            },
            ai,
            ai_runtime: AiRuntimeSave {
                behavior,
                timer_secs,
            },
            blocks,
        }
    }
}

/// Restore a saved behavior. Behaviors that carry a live entity target fall
/// back to `Idle`; the rule engine reacquires one on its next think tick.
fn restore_behavior(commands: &mut Commands, entity: Entity, label: &str, timer_secs: f32) {
    let behavior = match label {
        "Wander" => AiBehavior::Wander { point: Vec2::ZERO },
        "Patrol" => AiBehavior::Patrol {
            around: Vec2::ZERO,
            radius: 400.0,
        },
        "Guard" => AiBehavior::Guard {
            target: None,
            radius: 400.0,
        },
        "Follow" => AiBehavior::Follow {
            target: None,
            radius: 400.0,
        },
        "Regroup" => AiBehavior::Regroup {
            position: Vec2::ZERO,
            radius: 400.0,
        },
        "Kiting" => AiBehavior::Kite {
            target: None,
            range: 400.0,
        },
        "Flee" => AiBehavior::Flee {
            from: None,
            distance: 1000.0,
        },
        _ => AiBehavior::Idle,
    };

    // The think timer is recreated from the profile on spawn; only carry the
    // remaining delay so restored ships do not all think on the same tick.
    let duration = timer_secs.max(f32::EPSILON);
    let timer = Timer::from_seconds(duration, TimerMode::Repeating);
    commands.entity(entity).insert(AiRuntime {
        current: behavior,
        spawn_position: Vec2::ZERO,
        leader: None,
        rule_cooldowns: Vec::new(),
        think_timer: timer,
    });
}

impl RestoreSnapshot for ShipSnapshot {
    fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
        _team: Team,
        _parent: Option<Entity>,
    ) -> Result<Entity, DocumentError> {
        let placements: Vec<BlockPlacement> = self
            .blocks
            .iter()
            .map(|block| block.placement.clone())
            .collect();
        let design = ShipDesign {
            version: 1,
            id: self.id,
            name: String::new(),
            blocks: placements,
            ai: self.ai.clone(),
        };
        let team = Team::new(self.team);
        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));

        let Some((ship, block_entities)) =
            design.spawn(commands, registry, team, transform, config)
        else {
            return Err(DocumentError::Parse(format!(
                "saved ship {} has no core",
                self.id
            )));
        };

        commands.entity(ship).insert(Velocity {
            linear: Vec2::new(self.velocity.x, self.velocity.y),
            angular: self.velocity.angvel,
        });
        restore_behavior(
            commands,
            ship,
            &self.ai_runtime.behavior,
            self.ai_runtime.timer_secs,
        );

        for (entity, snapshot) in block_entities.iter().zip(self.blocks.iter()) {
            snapshot.clone().apply_to(commands, *entity);
        }

        Ok(ship)
    }
}

impl CaptureSnapshot for WorldSave {
    fn capture(ctx: &CaptureContext, _entity: Entity) -> Self {
        let world = ctx.world;

        let ships: Vec<ShipSnapshot> = world
            .iter_entities()
            .filter(|entity| entity.get::<Ship>().is_some() && entity.get::<ShipGrid>().is_some())
            .map(|entity| ShipSnapshot::capture(ctx, entity.id()))
            .collect();

        let drops: Vec<DropSave> = world
            .iter_entities()
            .filter_map(|entity| {
                let transform = entity.get::<Transform>()?;
                let drop = entity.get::<DropComponent>()?;
                let lifetime = entity.get::<Lifetime>()?;
                match drop {
                    DropComponent::Item(item) => Some(DropSave {
                        position: transform.translation.truncate(),
                        item_type: item.item_type.clone(),
                        amount: item.amount,
                        remaining_secs: lifetime.timer().remaining_secs(),
                    }),
                }
            })
            .collect();

        let camera: CameraSave = world
            .iter_entities()
            .filter_map(|entity| {
                let transform = entity.get::<Transform>()?;
                let projection = entity.get::<Projection>()?;
                match projection {
                    Projection::Orthographic(ortho) => Some(CameraSave {
                        x: transform.translation.x,
                        y: transform.translation.y,
                        zoom: ortho.scale,
                    }),
                    _ => None,
                }
            })
            .next()
            .unwrap_or_default();

        Self {
            version: SAVE_SCHEMA_VERSION,
            slot: ctx.slot,
            saved_at_unix_ms: ctx.saved_at_unix_ms,
            battle: BattleSave {
                mission: ctx.mission.to_string(),
                elapsed_secs: ctx.elapsed_secs,
                outcome: ctx.outcome.clone(),
            },
            camera,
            ships,
            drops,
            rng: RngSave { seed: ctx.seed },
        }
    }
}

impl RestoreSnapshot for WorldSave {
    fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
        team: Team,
        _parent: Option<Entity>,
    ) -> Result<Entity, DocumentError> {
        for ship in self.ships {
            ship.restore(commands, registry, config, team, None)?;
        }

        for drop in self.drops {
            let duration = drop.remaining_secs.max(0.0);
            commands.spawn((
                DropComponent::Item(ItemDrop {
                    item_type: drop.item_type,
                    amount: drop.amount,
                }),
                DespawnOnExit(SceneState::Game),
                Transform::from_translation(drop.position.extend(LAYER_DROP)),
                Lifetime::seconds(duration),
            ));
        }

        Ok(Entity::PLACEHOLDER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::content::block::{BlockShape, ItemDrop};

    fn registry() -> BlockRegistry {
        let mut registry = BlockRegistry::default();
        for (key, group, mass, health) in [
            ("core", "core", 50.0, 5000.0),
            ("plate", "base", 10.0, 1000.0),
        ] {
            registry.register(
                key,
                crate::content::block::BlockDef {
                    name: key.to_string(),
                    block_type: key.to_string(),
                    block_group: group.to_string(),
                    mass,
                    health,
                    shape: BlockShape::default(),
                    cost: 1,
                    features: Vec::new(),
                    drops: vec![DropComponent::Item(ItemDrop {
                        item_type: "steel".to_string(),
                        amount: 10,
                    })],
                },
            );
        }
        registry
    }

    #[test]
    fn embedded_world_snapshot_roundtrip() {
        let design = ShipDesign::new(vec![
            BlockPlacement::new(IVec2::ZERO, "core", 0),
            BlockPlacement::new(IVec2::X, "plate", 0),
        ]);

        let mut source = App::new();
        source
            .insert_resource(registry())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(Startup, {
                let design = design.clone();
                move |mut commands: Commands, reg: Res<BlockRegistry>, cfg: Res<GameConfigValue>| {
                    design.spawn(
                        &mut commands,
                        &reg,
                        Team::new(0),
                        Transform::from_xyz(12.0, -4.0, 0.0),
                        &cfg,
                    );
                }
            });
        source.update();

        let ctx = CaptureContext {
            world: source.world(),
            slot: 3,
            mission: "roundtrip-test",
            elapsed_secs: 17.5,
            outcome: Some("Victory".to_string()),
            saved_at_unix_ms: 1_700_000_000_000,
            seed: 4242,
        };
        let save = WorldSave::capture(&ctx, Entity::PLACEHOLDER);
        assert_eq!(save.slot, 3);
        assert_eq!(save.ships.len(), 1);
        assert_eq!(save.ships[0].blocks.len(), 2);
        assert_eq!(save.battle.elapsed_secs, 17.5);
        assert_eq!(save.ships[0].transform.x, 12.0);

        let mut restored = App::new();
        let restore_save = save.clone();
        restored
            .insert_resource(registry())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(
                Startup,
                move |mut commands: Commands,
                      reg: Res<BlockRegistry>,
                      cfg: Res<GameConfigValue>| {
                    restore_save
                        .clone()
                        .restore(&mut commands, &reg, &cfg, Team::new(0), None)
                        .expect("snapshot restores");
                },
            );
        restored.update();

        let mut grids = restored.world_mut().query::<&ShipGrid>();
        assert_eq!(grids.iter(restored.world()).count(), 1);
        let grid = grids.single(restored.world()).unwrap();
        assert_eq!(grid.blocks.len(), 2);
    }
}
