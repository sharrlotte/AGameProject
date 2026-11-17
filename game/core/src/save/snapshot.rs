//! `CaptureSnapshot`/`RestoreSnapshot` implementations for save data.
//!
//! Runtime ECS state is converted into serializable snapshots here, never in
//! the IO layer. Entity references are never persisted: turret targets, laser
//! beams and AI targets are reacquired after a load.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::turret::TurretState;
use crate::content::block::{
    BlockPlacement, BlockRef, BlockRegistry, BurstShootPatternState, DropComponent, Health,
    LaserBeamPatternState, PullingBeamPatternState, SingleShootPatternState, spawn_block_entity,
};
use crate::content::lifetime::Lifetime;
use crate::content::ship::ai::{
    AiProfile, BehaviorAttack, BehaviorChase, BehaviorFlee, BehaviorFollow, BehaviorGuard,
    BehaviorIdle, BehaviorKamikaze, BehaviorKiting, BehaviorPatrol, BehaviorRegroup, BehaviorTimer,
    BehaviorWander, behavior_label,
};
use crate::content::ship::{Ship, ShipDesign, ShipGrid};
use crate::content::team::Team;
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
        let turret = world
            .get::<TurretState>(entity)
            .map(|s| TurretSaveState {
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
                DocumentError::Parse(format!(
                    "unknown block type {}",
                    self.placement.block_type
                ))
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
        let ai = world.get::<AiProfile>(entity).copied().unwrap_or_default();

        let blocks = world
            .get::<ShipGrid>(entity)
            .map(|grid| {
                grid.blocks()
                    .filter(|b| b.entity != Entity::PLACEHOLDER)
                    .map(|b| BlockSnapshot::capture(ctx, b.entity))
                    .collect()
            })
            .unwrap_or_default();

        let behavior = behavior_label(
            world.get::<BehaviorIdle>(entity),
            world.get::<BehaviorWander>(entity),
            world.get::<BehaviorKiting>(entity),
            world.get::<BehaviorRegroup>(entity),
            world.get::<BehaviorChase>(entity),
            world.get::<BehaviorAttack>(entity),
            world.get::<BehaviorPatrol>(entity),
            world.get::<BehaviorFlee>(entity),
            world.get::<BehaviorGuard>(entity),
            world.get::<BehaviorFollow>(entity),
            world.get::<BehaviorKamikaze>(entity),
        )
        .to_string();
        let timer_secs = world
            .get::<BehaviorTimer>(entity)
            .map(|t| t.0.remaining_secs())
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

/// Insert the runtime marker matching a saved behavior label. Behaviors that
/// carry a live entity target fall back to `Idle`; the AI reacquires a target
/// on its next think tick.
fn restore_behavior(commands: &mut Commands, entity: Entity, label: &str, timer_secs: f32) {
    let mut ec = commands.entity(entity);
    ec.remove::<(
        BehaviorIdle,
        BehaviorWander,
        BehaviorKiting,
        BehaviorRegroup,
        BehaviorChase,
        BehaviorAttack,
        BehaviorPatrol,
        BehaviorFlee,
        BehaviorGuard,
        BehaviorFollow,
        BehaviorKamikaze,
    )>();

    let _ = timer_secs; // Think timer is recreated from config on spawn.
    match label {
        "Wander" => {
            ec.insert(BehaviorWander);
        }
        "Kiting" => {
            ec.insert(BehaviorKiting);
        }
        "Regroup" => {
            ec.insert(BehaviorRegroup);
        }
        "Patrol" => {
            ec.insert(BehaviorPatrol {
                around_position: Vec2::ZERO,
                radius: 400.0,
            });
        }
        _ => {
            ec.insert(BehaviorIdle);
        }
    }
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
            ai: self.ai,
        };
        let team = Team::new(self.team);
        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));

        let Some((ship, block_entities)) = design.spawn(commands, registry, team, transform, config)
        else {
            return Err(DocumentError::Parse(format!(
                "saved ship {} has no core",
                self.id
            )));
        };

        commands.entity(ship).insert((
            Velocity {
                linear: Vec2::new(self.velocity.x, self.velocity.y),
                angular: self.velocity.angvel,
            },
            self.ai,
        ));
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
            .filter(|entity| {
                entity.get::<Ship>().is_some() && entity.get::<ShipGrid>().is_some()
            })
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
                DropComponent::Item(crate::content::block::ItemDrop {
                    item_type: drop.item_type,
                    amount: drop.amount,
                }),
                DespawnOnExit(crate::resource::SceneState::Game),
                Transform::from_translation(drop.position.extend(crate::constant::LAYER_DROP)),
                Lifetime::seconds(duration),
            ));
        }

        Ok(Entity::PLACEHOLDER)
    }
}
