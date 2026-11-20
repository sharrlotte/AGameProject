use bevy::prelude::*;
use uuid::Uuid;

use crate::content::block::Loot;
use crate::content::body::BlockGrid;
use crate::content::lifetime::Lifetime;
use crate::content::ship::Ship;
use crate::content::ship::ai::{Brain, Doctrine, Skill, TacticalBoard, Tactic};
use crate::content::ship::stat::ShipStats;
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlMode, ControlledShip};

use super::BodySnapshot;
use super::{
    BlockSnapshot, BrainSave, CameraSave, CargoSave, ControlSave, DebrisSnapshot, DropSave,
    LastSeenSave, ShipSnapshot, TacticSave, TeamAlertSave, TeamIntelSave, TransformSave,
    VelocitySave,
};

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
}

impl ShipSnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        let ship = entity.get::<Ship>()?;
        let grid = entity.get::<BlockGrid>()?;
        let team = entity.get::<Team>().map(|t| t.id()).unwrap_or(0);
        let transform = entity.get::<Transform>().cloned().unwrap_or_default();
        let velocity = entity
            .get::<bevy_rapier2d::prelude::Velocity>()
            .cloned()
            .unwrap_or_default();
        let doctrine = entity.get::<Doctrine>().copied().unwrap_or_default();
        let skill = entity.get::<Skill>().map_or(5, |s| s.level());
        let cargo = entity
            .get::<ShipStats>()
            .map(|stats| CargoSave {
                used: stats.cargo_used(),
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
}

impl DebrisSnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        let grid = entity.get::<BlockGrid>()?;
        let team = entity.get::<Team>().map(|t| t.id()).unwrap_or(0);
        let transform = entity.get::<Transform>().cloned().unwrap_or_default();
        let velocity = entity
            .get::<bevy_rapier2d::prelude::Velocity>()
            .cloned()
            .unwrap_or_default();
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
