use bevy::prelude::*;
use rand::RngExt;
use std::collections::HashMap;

use super::component::*;
use crate::config::GameConfigValue;
use crate::content::ship::{BoundingBox, Ship, ShipBehavior, ShipStats};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlledShip, GameRng, SceneState};
use crate::simulation::SimulationSet;

/// Snapshot of a ship used by behavior systems; avoids borrowing ECS queries
/// across systems and keeps the team/enemy index rebuild in one place.
#[derive(Clone, Copy, Debug)]
pub struct ShipInfo {
    pub entity: Entity,
    pub position: Vec2,
    pub size_len: f32,
}

#[derive(Resource, Default)]
pub struct ShipIndex {
    pub teams: HashMap<u8, Vec<ShipInfo>>,
    pub enemies: HashMap<u8, Vec<ShipInfo>>,
}

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipIndex>().add_systems(
            bevy::prelude::FixedUpdate,
            (
                build_ship_index,
                idle_behavior,
                chase_behavior,
                wander_behavior,
                kiting_behavior,
                regroup_behavior,
                guard_behavior,
                follow_behavior,
                kamikaze_behavior,
                attack_behavior,
                patrol_behavior,
                flee_behavior,
            )
                .chain()
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Ai),
        );
    }
}

fn should_skip(entity: Entity, camera: &CameraTarget, controlled: &ControlledShip) -> bool {
    camera.following == Some(entity) || controlled.controlling == Some(entity)
}

fn in_range(index: &ShipIndex, team: u8, position: Vec2, max_range: f32) -> Vec<ShipInfo> {
    index
        .enemies
        .get(&team)
        .map(|list| {
            list.iter()
                .copied()
                .filter(|other| other.position.distance(position) <= max_range)
                .collect()
        })
        .unwrap_or_default()
}

pub fn build_ship_index(
    ships: Query<(Entity, &GlobalTransform, &BoundingBox, &Team)>,
    mut index: ResMut<ShipIndex>,
) {
    index.teams.clear();
    for (entity, transform, bbox, team) in &ships {
        index.teams.entry(team.id).or_default().push(ShipInfo {
            entity,
            position: transform.translation().truncate(),
            size_len: bbox.size.length(),
        });
    }

    index.enemies.clear();
    let team_ids: Vec<u8> = index.teams.keys().copied().collect();
    for team_id in team_ids {
        let mut enemies = Vec::new();
        for (other_id, list) in &index.teams {
            if *other_id != team_id {
                enemies.extend(list.iter().copied());
            }
        }
        index.enemies.insert(team_id, enemies);
    }
}


fn tick(
    timer: &mut BehaviorTimer,
    time: &Time,
    entity: Entity,
    camera: &CameraTarget,
    controlled: &ControlledShip,
) -> bool {
    if !timer.0.tick(time.delta()).just_finished() {
        return false;
    }
    !should_skip(entity, camera, controlled)
}

pub fn idle_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &Ship,
            &ShipStats,
            &BoundingBox,
            &Team,
        ),
        With<BehaviorIdle>,
    >,
    index: Res<ShipIndex>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut commands: Commands,
) {
    for (entity, mut timer, transform, _ship, stats, bbox, team) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let range = in_range(&index, team.id, transform.translation.truncate(), stats.max_range);
        if let Some(enemy) = range.first() {
            set_behavior!(commands, entity, BehaviorChase { target: enemy.entity });
        } else {
            commands.entity(entity).insert(ShipBehavior::Approach {
                position: Vec2::ZERO,
                radius: bbox.size.length() * config.0.ai_group_range_multiplier,
            });
        }
    }
}

pub fn chase_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &Ship,
            &ShipStats,
        ),
        With<BehaviorChase>,
    >,
    chases: Query<&BehaviorChase>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut commands: Commands,
) {
    for (entity, mut timer, transform, _ship, stats) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let target = chases.get(entity).map(|b| b.target).unwrap_or(entity);
        if target == entity {
            warn!("AI ship {:?} had itself as chase target, idling", entity);
            set_behavior!(commands, entity, BehaviorIdle);
            continue;
        }

        let Ok(target_transform) = transforms.get(target) else {
            set_behavior!(commands, entity, BehaviorIdle);
            continue;
        };
        let position = target_transform.translation.truncate();
        if transform.translation.distance(position.extend(0.0)) >= stats.min_range {
            commands.entity(entity).insert(ShipBehavior::Approach {
                position,
                radius: stats.min_range * config.0.ai_min_range_factor,
            });
        } else {
            commands.entity(entity).insert(ShipBehavior::GoAround {
                position,
                radius: stats.min_range * config.0.ai_min_range_factor,
            });
        }
    }
}

pub fn wander_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &Ship,
            &ShipStats,
            &BoundingBox,
        ),
        With<BehaviorWander>,
    >,
    index: Res<ShipIndex>,
    teams: Query<&Team>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut game_rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    let rng = game_rng.rng_mut();
    for (entity, mut timer, transform, _ship, stats, bbox) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(team) = teams.get(entity) else {
            continue;
        };
        let allies = index
            .teams
            .get(&team.id)
            .map(|v| v.as_slice())
            .unwrap_or_default();
        let ally = allies.iter().rev().find(|a| a.entity != entity).copied();
        let has_enemy = !in_range(
            &index,
            team.id,
            transform.translation.truncate(),
            stats.max_range,
        )
        .is_empty();

        if rng.random_bool(config.0.ai_wander_attack_chance as f64) && has_enemy {
            set_behavior!(commands, entity, BehaviorKiting);
        } else if rng.random_bool(config.0.ai_wander_guard_chance as f64) {
            set_behavior!(
                commands,
                entity,
                BehaviorAttack {
                    target_position: Vec2::ZERO,
                }
            );
        } else if rng.random_bool(config.0.ai_wander_guard_chance as f64) {
            if let Some(a) = ally {
                set_behavior!(
                    commands,
                    entity,
                    BehaviorGuard {
                        target: a.entity,
                        radius: (a.size_len + bbox.size.length())
                            * config.0.ai_group_range_multiplier,
                    }
                );
            }
        } else {
            set_behavior!(
                commands,
                entity,
                BehaviorPatrol {
                    around_position: Vec2::ZERO,
                    radius: bbox.size.length() * config.0.ai_wander_patrol_radius_multiplier,
                }
            );
        }
    }
}

pub fn kiting_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &Ship,
            &ShipStats,
        ),
        With<BehaviorKiting>,
    >,
    index: Res<ShipIndex>,
    teams: Query<&Team>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut commands: Commands,
) {
    for (entity, mut timer, transform, _ship, stats) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(team) = teams.get(entity) else {
            continue;
        };
        let desired = stats.max_range * config.0.ai_min_range_factor;
        let position = transform.translation.truncate();

        let nearest = index
            .enemies
            .get(&team.id)
            .map(|list| {
                list.iter()
                    .copied()
                    .filter(|other| other.position.distance(position) <= stats.max_range)
                    .min_by(|a, b| {
                        a.position
                            .distance_squared(position)
                            .total_cmp(&b.position.distance_squared(position))
                    })
            })
            .flatten();

        match nearest {
            Some(enemy) => {
                let distance = enemy.position.distance(position);
                if distance < desired {
                    let away = (position - enemy.position).normalize_or_zero();
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: enemy.position + away * stats.max_range,
                        radius: desired,
                    });
                } else {
                    commands.entity(entity).insert(ShipBehavior::GoAround {
                        position: enemy.position,
                        radius: desired,
                    });
                }
            }
            None => set_behavior!(commands, entity, BehaviorIdle),
        }
    }
}

pub fn regroup_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &BoundingBox,
        ),
        With<BehaviorRegroup>,
    >,
    index: Res<ShipIndex>,
    teams: Query<&Team>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut commands: Commands,
) {
    for (entity, mut timer, transform, bbox) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(team) = teams.get(entity) else {
            continue;
        };
        let ally = index
            .teams
            .get(&team.id)
            .and_then(|list| list.iter().rev().find(|a| a.entity != entity).copied());

        if let Some(a) = ally {
            let in_range = transform.translation.truncate().distance(a.position)
                <= (a.size_len + bbox.size.length()) * config.0.ai_group_range_multiplier;
            if in_range {
                set_behavior!(commands, entity, BehaviorWander);
            } else {
                let target = transforms
                    .get(a.entity)
                    .map(|t| t.transform_point((Vec2::ZERO).extend(0.0)).truncate())
                    .unwrap_or(a.position);
                commands.entity(entity).insert(ShipBehavior::Approach {
                    position: target,
                    radius: (a.size_len + bbox.size.length())
                        * config.0.ai_group_range_multiplier,
                });
            }
        } else {
            set_behavior!(commands, entity, BehaviorIdle);
        }
    }
}

pub fn guard_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
        ),
        With<BehaviorGuard>,
    >,
    guards: Query<&BehaviorGuard>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    mut commands: Commands,
) {
    for (entity, mut timer, _) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(guard) = guards.get(entity) else {
            continue;
        };
        if let Ok(position) = transforms.get(guard.target) {
            commands.entity(entity).insert(ShipBehavior::Approach {
                position: position.translation.truncate(),
                radius: guard.radius,
            });
        } else {
            set_behavior!(commands, entity, BehaviorIdle);
        }
    }
}

pub fn follow_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
        ),
        With<BehaviorFollow>,
    >,
    follows: Query<&BehaviorFollow>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(follow) = follows.get(entity) else {
            continue;
        };
        if let Ok(position) = transforms.get(follow.target) {
            commands.entity(entity).insert(ShipBehavior::Approach {
                position: position.translation.truncate(),
                radius: follow.radius,
            });
        } else {
            set_behavior!(commands, entity, BehaviorIdle);
        }
    }
}

pub fn kamikaze_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &BoundingBox,
        ),
        With<BehaviorKamikaze>,
    >,
    kamikazes: Query<&BehaviorKamikaze>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    mut commands: Commands,
) {
    for (entity, mut timer, bbox) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(kamikaze) = kamikazes.get(entity) else {
            continue;
        };
        if let Ok(position) = transforms.get(kamikaze.target) {
            commands.entity(entity).insert(ShipBehavior::Approach {
                position: position.translation.truncate(),
                radius: bbox.size.length(),
            });
        } else {
            set_behavior!(commands, entity, BehaviorIdle);
        }
    }
}

pub fn attack_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &BoundingBox,
        ),
        With<BehaviorAttack>,
    >,
    attacks: Query<&BehaviorAttack>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    mut commands: Commands,
) {
    for (entity, mut timer, bbox) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(attack) = attacks.get(entity) else {
            continue;
        };
        commands.entity(entity).insert(ShipBehavior::Approach {
            position: attack.target_position,
            radius: bbox.size.length(),
        });
    }
}

pub fn patrol_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
        ),
        With<BehaviorPatrol>,
    >,
    patrols: Query<&BehaviorPatrol>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(patrol) = patrols.get(entity) else {
            continue;
        };
        commands.entity(entity).insert(ShipBehavior::Approach {
            position: patrol.around_position,
            radius: patrol.radius,
        });
    }
}

pub fn flee_behavior(
    time: Res<Time>,
    mut ships: Query<
        (
            Entity,
            &mut BehaviorTimer,
            &Transform,
            &BoundingBox,
        ),
        With<BehaviorFlee>,
    >,
    flees: Query<&BehaviorFlee>,
    transforms: Query<&Transform>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    config: Res<GameConfigValue>,
    mut commands: Commands,
) {
    for (entity, mut timer, transform, bbox) in &mut ships {
        if !tick(&mut timer, &time, entity, &camera, &controlled) {
            continue;
        }
        let Ok(flee) = flees.get(entity) else {
            continue;
        };
        if let Ok(position) = transforms.get(flee.from) {
            let direction =
                (transform.translation.truncate() - position.translation.truncate())
                    .normalize_or_zero();
            let op_position = transform.translation.truncate()
                + (direction * -1.0) * config.0.ai_flee_distance;
            commands.entity(entity).insert(ShipBehavior::Approach {
                position: op_position,
                radius: bbox.size.length(),
            });
        }
    }
}
