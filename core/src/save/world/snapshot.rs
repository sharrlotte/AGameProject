use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::constant::LAYER_DROP;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::power::{Booster, Charge};
use crate::content::block::feature::turret::{TurretBarrel, TurretCharge, TurretState};
use crate::content::block::{
    BlockPlacement, BlockRegistry, BurstShootPatternState, Disabled, GridBlock, Health, ItemDrop,
    LaserBeamPatternShootingState, LaserBeamPatternState, Loot, PullingBeamPatternState,
    SingleShootPatternState,
    spawn_block_entity,
};
use crate::content::body::debris::{component_mass, spawn_debris_body};
use crate::content::body::{BlockGrid, Debris};
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::ship::ai::{
    Brain, Doctrine, LastSeen, Posture, Skill, SupportKind, Tactic, TacticalBoard, TeamAlert,
    TeamPlan,
};
use crate::content::ship::stat::ShipStats;
use crate::content::ship::{Ship, ShipDesign};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlMode, ControlledShip, SceneState};
use crate::save::design::DESIGN_SCHEMA_VERSION;
use crate::save::world::component::WorldStoreError;

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

impl ShipSnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        let ship = entity.get::<Ship>()?;
        let grid = entity.get::<BlockGrid>()?;
        let team = entity.get::<Team>().map(|t| t.id).unwrap_or(0);
        let transform = entity.get::<Transform>().cloned().unwrap_or_default();
        let velocity = entity.get::<Velocity>().cloned().unwrap_or_default();
        let doctrine = entity.get::<Doctrine>().copied().unwrap_or_default();
        let skill = entity.get::<Skill>().map_or(5, |s| s.level());
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

        let brain = entity
            .get::<Brain>()
            .map(|b| BrainSave {
                current: TacticSave::from_runtime(world, &b.current),
                spawn_position: b.spawn_position,
                timer_secs: b.think_timer.duration().as_secs_f32(),
                strafe_sign: b.strafe_sign,
                strafe_timer: b.strafe_timer,
                shed_boosters: b.shed_boosters,
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
            doctrine,
            skill,
            brain,
            blocks,
        })
    }

    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
    ) -> Result<(Uuid, Entity, u8, BrainSave), WorldStoreError> {
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
            doctrine: self.doctrine,
        };
        let team = Team::new(self.team);
        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));

        let Some((ship, block_entities)) =
            design.spawn_with_id(commands, registry, team, transform, config, self.id)
        else {
            return Err(WorldStoreError::InvalidData(format!(
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
            if self.brain.shed_boosters {
                commands.entity(*entity).queue(|mut block: EntityWorldMut| {
                    if block.get::<Booster>().is_some() {
                        block.insert(Disabled);
                    }
                });
            }
        }

        Ok((self.id, ship, self.skill, self.brain))
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

pub fn capture_camera(world: &World) -> CameraSave {
    let following = world
        .get_resource::<CameraTarget>()
        .and_then(|t| t.following)
        .and_then(|e| world.get::<Ship>(e))
        .map(|s| s.id);

    world
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
        })
}

pub fn capture_control(world: &World) -> ControlSave {
    let controlling = world
        .get_resource::<ControlledShip>()
        .and_then(|c| c.controlling)
        .and_then(|e| world.get::<Ship>(e))
        .map(|s| s.id);
    let pilot_mode = world
        .get_resource::<ControlMode>()
        .is_some_and(|m| *m == ControlMode::Pilot);

    ControlSave {
        controlling,
        pilot_mode,
    }
}

pub fn capture_bodies(world: &World) -> Vec<BodySnapshot> {
    world
        .iter_entities()
        .filter_map(|entity| BodySnapshot::capture(world, entity))
        .collect()
}

pub fn capture_drops(world: &World) -> Vec<DropSave> {
    world
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
        .collect()
}

pub fn capture_intel(world: &World) -> Vec<TeamIntelSave> {
    world
        .get_resource::<TacticalBoard>()
        .map(|board| {
            board
                .teams
                .iter()
                .map(|(&team, plan)| TeamIntelSave {
                    team,
                    last_known: plan
                        .intel
                        .last_known
                        .iter()
                        .filter_map(|(entity, seen)| {
                            let target_id = world.get::<Ship>(*entity).map(|s| s.id)?;
                            Some(LastSeenSave {
                                target_id,
                                position: seen.position,
                                velocity: seen.velocity,
                                ttl: seen.ttl,
                            })
                        })
                        .collect(),
                    alerts: plan
                        .intel
                        .alerts
                        .iter()
                        .map(|a| TeamAlertSave {
                            position: a.position,
                            ttl: a.ttl,
                        })
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn apply_bodies(
    commands: &mut Commands,
    bodies: Vec<BodySnapshot>,
    registry: &BlockRegistry,
    config: &GameConfigValue,
) -> Result<HashMap<Uuid, Entity>, WorldStoreError> {
    let (ship_bodies, debris_bodies): (Vec<BodySnapshot>, Vec<BodySnapshot>) = bodies
        .into_iter()
        .partition(|body| matches!(body, BodySnapshot::Ship(_)));

    let mut ship_map = HashMap::with_capacity(ship_bodies.len());
    let mut pending_ai = Vec::with_capacity(ship_bodies.len());

    for body in ship_bodies {
        let BodySnapshot::Ship(ship) = body else {
            continue;
        };
        let (id, entity, skill, brain) = ship.restore(commands, registry, config)?;
        ship_map.insert(id, entity);
        pending_ai.push((entity, skill, brain));
    }

    for body in debris_bodies {
        let BodySnapshot::Debris(debris) = body else {
            continue;
        };
        debris.restore(commands, registry, config);
    }

    for (entity, skill, brain) in pending_ai {
        let tactic = brain.current.into_runtime(&ship_map, brain.spawn_position);
        let duration = brain.timer_secs.max(0.05);
        let timer = Timer::from_seconds(duration, TimerMode::Repeating);
        commands.entity(entity).insert((
            Skill::new(skill),
            Brain {
                current: tactic,
                spawn_position: brain.spawn_position,
                think_timer: timer,
                strafe_sign: if brain.strafe_sign == 0.0 {
                    1.0
                } else {
                    brain.strafe_sign
                },
                strafe_timer: brain.strafe_timer.max(0.1),
                shed_boosters: brain.shed_boosters,
            },
        ));
    }

    Ok(ship_map)
}

pub fn apply_intel(
    commands: &mut Commands,
    intel: Vec<TeamIntelSave>,
    ship_map: &HashMap<Uuid, Entity>,
) {
    let mut board = TacticalBoard::default();
    for saved_intel in intel {
        let mut plan = TeamPlan::default();
        for seen in saved_intel.last_known {
            if let Some(&target_entity) = ship_map.get(&seen.target_id) {
                plan.intel.last_known.insert(
                    target_entity,
                    LastSeen {
                        position: seen.position,
                        velocity: seen.velocity,
                        ttl: seen.ttl,
                    },
                );
            }
        }
        for alert in saved_intel.alerts {
            plan.intel.alerts.push(TeamAlert {
                position: alert.position,
                ttl: alert.ttl,
            });
        }
        board.teams.insert(saved_intel.team, plan);
    }
    commands.insert_resource(board);
}

pub fn apply_control(
    commands: &mut Commands,
    control: &ControlSave,
    ship_map: &HashMap<Uuid, Entity>,
) {
    let controlling = control
        .controlling
        .and_then(|id| ship_map.get(&id).copied());
    let mode = if control.pilot_mode && controlling.is_some() {
        ControlMode::Pilot
    } else {
        ControlMode::FreeLook
    };
    commands.insert_resource(ControlledShip { controlling });
    commands.insert_resource(mode);
}

pub fn apply_camera(
    camera: &mut Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    saved: &CameraSave,
    config: &GameConfigValue,
    ship_map: &HashMap<Uuid, Entity>,
    commands: &mut Commands,
) {
    let following = saved.following.and_then(|id| ship_map.get(&id).copied());
    commands.insert_resource(CameraTarget { following });

    for (mut transform, mut projection) in camera {
        transform.translation.x = saved.x;
        transform.translation.y = saved.y;
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scale = saved.zoom.clamp(config.0.min_zoom, config.0.max_zoom);
        }
    }
}

pub fn apply_drops(
    commands: &mut Commands,
    drops: Vec<DropSave>,
    config: &GameConfigValue,
    items: Option<&ItemRegistry>,
) {
    for drop in drops {
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
}
