use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::constant::LAYER_DROP;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::power::Charge;
use crate::content::block::feature::turret::{TurretBarrel, TurretCharge, TurretState};
use crate::content::block::{
    BlockPlacement, BlockRegistry, BurstShootPatternState, GridBlock, Health, ItemDrop,
    LaserBeamPatternState, Loot, PullingBeamPatternState, SingleShootPatternState,
    spawn_block_entity,
};
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::ship::ai::{AiProfile, AiRuntime};
use crate::content::body::debris::{component_mass, spawn_debris_body};
use crate::content::body::{BlockGrid, Debris};
use crate::content::ship::stat::ShipStats;
use crate::content::ship::{Ship, ShipDesign};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlMode, ControlledShip, SceneState};
use crate::save::campaign::clear_campaign_save_in;
use crate::save::design::DESIGN_SCHEMA_VERSION;
use crate::save::document::{DocumentError, data_dir, remove_with_backup};

use super::component::{
    AiBehaviorSave, AiRuntimeSave, BattleSave, BlockSnapshot, BodySnapshot, CameraSave,
    CaptureContext, CargoSave, ControlSave, DebrisSnapshot, DropSave, RngSave, SAVE_SCHEMA_VERSION,
    ShipSnapshot, TransformSave, TurretSaveState, VelocitySave, WorldSave,
};

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

impl ShipSnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        let ship = entity.get::<Ship>()?;
        let grid = entity.get::<BlockGrid>()?;
        let team = entity.get::<Team>().map(|t| t.id).unwrap_or(0);
        let transform = entity.get::<Transform>().cloned().unwrap_or_default();
        let velocity = entity.get::<Velocity>().cloned().unwrap_or_default();
        let ai = entity.get::<AiProfile>().cloned().unwrap_or_default();
        let cargo = entity
            .get::<ShipStats>()
            .map(|stats| CargoSave {
                used: stats.cargo_used,
                items: stats
                    .cargo_items
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect(),
            })
            .unwrap_or_default();

        let blocks = grid
            .blocks()
            .filter(|b| b.entity != Entity::PLACEHOLDER)
            .map(|b| BlockSnapshot::capture(world, b))
            .collect();

        let ai_runtime = entity
            .get::<AiRuntime>()
            .map(|r| AiRuntimeSave {
                current: AiBehaviorSave::from_runtime(world, &r.current),
                spawn_position: r.spawn_position,
                rule_cooldowns: r.rule_cooldowns.clone(),
                timer_secs: r.think_timer.remaining_secs(),
            })
            .unwrap_or_default();

        Some(Self {
            id: ship.id,
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
            cargo,
            ai,
            ai_runtime,
            blocks,
        })
    }

    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
    ) -> Result<(Uuid, Entity, AiRuntimeSave), DocumentError> {
        let placements: Vec<BlockPlacement> = self
            .blocks
            .iter()
            .map(|block| block.placement.clone())
            .collect();
        let design = ShipDesign {
            version: DESIGN_SCHEMA_VERSION,
            id: self.id,
            name: String::new(),
            blocks: placements,
            ai: self.ai.clone(),
        };
        let team = Team::new(self.team);
        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));

        let Some((ship, block_entities)) =
            design.spawn_with_id(commands, registry, team, transform, config, self.id)
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

        let cargo = self.cargo;
        commands.entity(ship).queue(move |mut entity: EntityWorldMut| {
            if let Some(mut stats) = entity.get_mut::<ShipStats>() {
                let cap = stats.cargo_capacity;
                stats.cargo_used = cargo.used.min(cap);
                stats.cargo_items = cargo.items.into_iter().collect();
            }
        });

        for (entity, snapshot) in block_entities.iter().zip(self.blocks) {
            snapshot.apply(commands, *entity);
        }

        Ok((self.id, ship, self.ai_runtime))
    }
}

impl BodySnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        if entity.get::<Debris>().is_some() {
            return DebrisSnapshot::capture(world, entity).map(Self::Debris);
        }

        ShipSnapshot::capture(world, entity).map(Self::Ship)
    }
}

impl DebrisSnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        let grid = entity.get::<BlockGrid>()?;
        let team = entity.get::<Team>().map(|t| t.id).unwrap_or(0);
        let transform = entity.get::<Transform>().cloned().unwrap_or_default();
        let velocity = entity.get::<Velocity>().cloned().unwrap_or_default();
        let remaining_secs = entity
            .get::<Lifetime>()
            .map(|lifetime| lifetime.timer().remaining_secs())
            .unwrap_or_default();

        let blocks = grid
            .blocks()
            .filter(|block| block.entity != Entity::PLACEHOLDER)
            .map(|block| BlockSnapshot::capture(world, block))
            .collect();

        Some(Self {
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
            remaining_secs,
            blocks,
        })
    }

    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
    ) -> Entity {
        let placements: Vec<BlockPlacement> = self
            .blocks
            .iter()
            .map(|block| block.placement.clone())
            .collect();

        let mut grid = BlockGrid::new();
        grid.rebuild_from_layout(&placements, registry);

        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));
        let velocity = Velocity {
            linear: Vec2::new(self.velocity.x, self.velocity.y),
            angular: self.velocity.angvel,
        };
        let team = Team::new(self.team);
        let collider = registry.build_voxel_collider_2d(&grid);
        let mass = component_mass(&grid, registry);

        let body = spawn_debris_body(
            commands,
            grid,
            transform,
            velocity,
            team,
            self.remaining_secs,
            collider,
            mass,
            config,
        );

        let mut spawned = Vec::with_capacity(self.blocks.len());
        for snapshot in self.blocks {
            let Some(def) = registry.get_symbol(&snapshot.placement.block_type) else {
                continue;
            };
            let id = snapshot.placement.id;
            let block = spawn_block_entity(commands, &snapshot.placement, def, body, team, config);
            snapshot.apply(commands, block);
            spawned.push((id, block));
        }

        commands.entity(body).queue(move |mut entity: EntityWorldMut| {
            if let Some(mut grid) = entity.get_mut::<BlockGrid>() {
                for (id, block) in spawned {
                    grid.set_entity(id, block);
                }
            }
        });

        body
    }
}

impl WorldSave {
    pub fn capture(ctx: &CaptureContext) -> Self {
        let world = ctx.world;

        let bodies: Vec<BodySnapshot> = world
            .iter_entities()
            .filter_map(|entity| BodySnapshot::capture(world, entity))
            .collect();

        let drops: Vec<DropSave> = world
            .iter_entities()
            .filter_map(|entity| {
                let transform = entity.get::<Transform>()?;
                let drop = entity.get::<Loot>()?;
                let lifetime = entity.get::<Lifetime>()?;
                match drop {
                    Loot::Item(item) => Some(DropSave {
                        position: transform.translation.truncate(),
                        item_type: item.item_type.clone(),
                        amount: item.amount,
                        remaining_secs: lifetime.timer().remaining_secs(),
                    }),
                }
            })
            .collect();

        let following = world
            .get_resource::<CameraTarget>()
            .and_then(|t| t.following)
            .and_then(|e| world.get::<Ship>(e))
            .map(|s| s.id);

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
                        following,
                    }),
                    _ => None,
                }
            })
            .next()
            .unwrap_or(CameraSave {
                following,
                ..default()
            });

        let controlling = world
            .get_resource::<ControlledShip>()
            .and_then(|c| c.controlling)
            .and_then(|e| world.get::<Ship>(e))
            .map(|s| s.id);
        let pilot_mode = world
            .get_resource::<ControlMode>()
            .is_some_and(|m| *m == ControlMode::Pilot);

        Self {
            version: SAVE_SCHEMA_VERSION,
            saved_at_unix_ms: ctx.saved_at_unix_ms,
            battle: BattleSave {
                mission: ctx.mission.to_string(),
                elapsed_secs: ctx.elapsed_secs,
                outcome: ctx.outcome.clone(),
            },
            camera,
            control: ControlSave {
                controlling,
                pilot_mode,
            },
            bodies,
            drops,
            rng: RngSave { seed: ctx.seed },
        }
    }

    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
        items: Option<&ItemRegistry>,
    ) -> Result<HashMap<Uuid, Entity>, DocumentError> {
        let (ship_bodies, debris_bodies): (Vec<BodySnapshot>, Vec<BodySnapshot>) = self
            .bodies
            .into_iter()
            .partition(|body| matches!(body, BodySnapshot::Ship(_)));

        let mut ship_map = HashMap::with_capacity(ship_bodies.len());
        let mut pending_ai = Vec::with_capacity(ship_bodies.len());

        for body in ship_bodies {
            let BodySnapshot::Ship(ship) = body else {
                continue;
            };
            let (id, entity, ai_runtime) = ship.restore(commands, registry, config)?;
            ship_map.insert(id, entity);
            pending_ai.push((entity, ai_runtime));
        }

        for body in debris_bodies {
            let BodySnapshot::Debris(debris) = body else {
                continue;
            };
            debris.restore(commands, registry, config);
        }

        for (entity, ai_runtime) in pending_ai {
            let behavior = ai_runtime.current.into_runtime(&ship_map);
            let duration = ai_runtime.timer_secs.max(f32::EPSILON);
            let timer = Timer::from_seconds(duration, TimerMode::Repeating);
            commands.entity(entity).insert(AiRuntime {
                current: behavior,
                spawn_position: ai_runtime.spawn_position,
                rule_cooldowns: ai_runtime.rule_cooldowns,
                think_timer: timer,
            });
        }

        let following = self
            .camera
            .following
            .and_then(|id| ship_map.get(&id).copied());
        let controlling = self
            .control
            .controlling
            .and_then(|id| ship_map.get(&id).copied());
        let mode = if self.control.pilot_mode && controlling.is_some() {
            ControlMode::Pilot
        } else {
            ControlMode::FreeLook
        };
        commands.insert_resource(CameraTarget { following });
        commands.insert_resource(ControlledShip { controlling });
        commands.insert_resource(mode);

        for drop in self.drops {
            let duration = drop.remaining_secs.max(0.0);
            let color = items
                .and_then(|reg| reg.get(&drop.item_type))
                .map(|def| def.rarity.color())
                .unwrap_or(Color::srgb(5.0, 5.0, 5.0));
            commands.spawn((
                Loot::Item(ItemDrop {
                    item_type: drop.item_type,
                    amount: drop.amount,
                }),
                DespawnOnExit(SceneState::Game),
                Transform::from_translation(drop.position.extend(LAYER_DROP)),
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(config.0.drop_size, config.0.drop_size)),
                    ..default()
                },
                Lifetime::seconds(duration),
            ));
        }

        Ok(ship_map)
    }
}

pub fn battle_save_path() -> PathBuf {
    battle_save_path_in(&data_dir())
}

pub fn battle_save_path_in(base: &Path) -> PathBuf {
    base.join("saves").join("battle.ron")
}

pub fn has_battle_save() -> bool {
    battle_save_path().is_file()
}

pub fn clear_battle_save() {
    clear_battle_save_in(&data_dir());
}

pub fn clear_battle_save_in(base: &Path) {
    remove_with_backup(&battle_save_path_in(base));
}

pub fn reset_campaign_data() {
    reset_campaign_data_in(&data_dir());
}

pub fn reset_campaign_data_in(base: &Path) {
    clear_battle_save_in(base);
    clear_campaign_save_in(base);
}
