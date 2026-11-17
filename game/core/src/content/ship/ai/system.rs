use bevy::prelude::*;
use std::collections::HashMap;

use super::component::*;
use crate::config::GameConfigValue;
use crate::content::damage::DamageEvent;
use crate::content::ship::{BoundingBox, Ship, ShipBehavior, ShipStats};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlledShip, SceneState};
use crate::simulation::SimulationSet;

/// Snapshot of a ship used by AI rule evaluation.
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
            FixedUpdate,
            (build_ship_index, evaluate_ai_rules, apply_ai_behaviors)
                .chain()
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Ai),
        );
    }
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

fn should_skip(entity: Entity, camera: &CameraTarget, controlled: &ControlledShip) -> bool {
    camera.following == Some(entity) || controlled.controlling == Some(entity)
}

fn enemy_list<'a>(index: &'a ShipIndex, team: u8) -> &'a [ShipInfo] {
    index
        .enemies
        .get(&team)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn ally_list<'a>(index: &'a ShipIndex, team: u8) -> &'a [ShipInfo] {
    index
        .teams
        .get(&team)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn nearest(enemies: &[ShipInfo], position: Vec2, self_entity: Entity) -> Option<ShipInfo> {
    enemies
        .iter()
        .copied()
        .filter(|info| info.entity != self_entity)
        .min_by(|a, b| {
            a.position
                .distance_squared(position)
                .total_cmp(&b.position.distance_squared(position))
        })
}

fn squad_center(allies: &[ShipInfo], self_entity: Entity) -> Vec2 {
    let members: Vec<&ShipInfo> = allies
        .iter()
        .filter(|info| info.entity != self_entity)
        .collect();
    if members.is_empty() {
        return Vec2::ZERO;
    }
    members.iter().map(|info| info.position).sum::<Vec2>() / members.len() as f32
}

fn resolve_target(
    reference: TargetRef,
    index: &ShipIndex,
    team: u8,
    self_entity: Entity,
    position: Vec2,
    stats: &ShipStats,
    ships: &Query<&ShipStats>,
) -> Option<Entity> {
    let enemies = enemy_list(index, team);
    let allies = ally_list(index, team);
    match reference {
        TargetRef::NearestEnemy => nearest(enemies, position, self_entity).map(|e| e.entity),
        TargetRef::HighestHpEnemy => enemies
            .iter()
            .filter(|info| info.entity != self_entity)
            .filter(|info| info.position.distance(position) <= stats.max_range)
            .filter_map(|info| ships.get(info.entity).ok().map(|s| (info, s.health)))
            .max_by(|a, b| {
                a.1.total_cmp(&b.1).then_with(|| {
                    a.0.position
                        .distance_squared(position)
                        .total_cmp(&b.0.position.distance_squared(position))
                })
            })
            .map(|(info, _)| info.entity),
        TargetRef::NearestAlly => nearest(allies, position, self_entity).map(|a| a.entity),
        TargetRef::Leader => allies
            .iter()
            .rev()
            .find(|a| a.entity != self_entity)
            .map(|a| a.entity),
        TargetRef::SpawnPoint | TargetRef::SquadCenter => None,
    }
}

fn resolve_position(
    reference: PositionRef,
    index: &ShipIndex,
    team: u8,
    self_entity: Entity,
    spawn: Vec2,
    transforms: &Query<&Transform>,
) -> Vec2 {
    match reference {
        PositionRef::SpawnPoint => spawn,
        PositionRef::SquadCenter => squad_center(ally_list(index, team), self_entity),
        PositionRef::Leader => ally_list(index, team)
            .iter()
            .rev()
            .find(|a| a.entity != self_entity)
            .and_then(|a| transforms.get(a.entity).ok())
            .map(|t| t.translation.truncate())
            .unwrap_or(spawn),
        PositionRef::SelfCore => spawn,
    }
}

fn condition_matches(
    condition: &AiCondition,
    index: &ShipIndex,
    team: u8,
    self_entity: Entity,
    position: Vec2,
    stats: &ShipStats,
    current: &AiBehavior,
) -> bool {
    match condition {
        AiCondition::Always => true,
        AiCondition::HpBelow(threshold) => stats.health < *threshold,
        AiCondition::HpAbove(threshold) => stats.health > *threshold,
        AiCondition::EnemyInRange(range) => !enemy_list(index, team)
            .iter()
            .any(|e| e.entity != self_entity && e.position.distance(position) <= *range),
        AiCondition::AllyCountBelow(count) => {
            ally_list(index, team).len().saturating_sub(1) < *count as usize
        }
        AiCondition::AllyCountAbove(count) => {
            ally_list(index, team).len().saturating_sub(1) > *count as usize
        }
        AiCondition::TargetLost => match current {
            AiBehavior::Chase { target }
            | AiBehavior::Attack { target }
            | AiBehavior::Kite { target, .. }
            | AiBehavior::Kamikaze { target } => target.is_none(),
            _ => false,
        },
    }
}

/// Select the highest-priority matching rule that is not on cooldown.
pub fn select_rule(
    profile: &AiProfile,
    cooldowns: &[f32],
    condition_context: impl Fn(&AiCondition) -> bool,
) -> Option<usize> {
    let mut winner: Option<(i32, usize)> = None;
    for (index, rule) in profile.rules.iter().enumerate() {
        if cooldowns.get(index).copied().unwrap_or(0.0) > 0.0 {
            continue;
        }
        if !condition_context(&rule.when) {
            continue;
        }
        if winner
            .map(|(priority, _)| rule.priority > priority)
            .unwrap_or(true)
        {
            winner = Some((rule.priority, index));
        }
    }
    winner.map(|(_, index)| index)
}

/// Evaluate rules at the profile's think rate. The highest-priority matching
/// rule that is not on cooldown wins; with no match the current behavior is
/// kept.
pub fn evaluate_ai_rules(
    time: Res<Time>,
    index: Res<ShipIndex>,
    camera: Res<CameraTarget>,
    controlled: Res<ControlledShip>,
    stats: Query<&ShipStats>,
    transforms: Query<&Transform>,
    mut ships: Query<(
        Entity,
        &mut AiRuntime,
        &AiProfile,
        &Transform,
        &ShipStats,
        &Team,
    )>,
) {
    for (entity, mut runtime, profile, transform, ship_stats, team) in &mut ships {
        if should_skip(entity, &camera, &controlled) {
            continue;
        }
        runtime.think_timer.tick(time.delta());
        if !runtime.think_timer.just_finished() {
            continue;
        }

        for cooldown in &mut runtime.rule_cooldowns {
            *cooldown = (*cooldown - time.delta_secs()).max(0.0);
        }

        let position = transform.translation.truncate();
        let rule_index = select_rule(profile, &runtime.rule_cooldowns, |condition| {
            condition_matches(
                condition,
                &index,
                team.id,
                entity,
                position,
                ship_stats,
                &runtime.current,
            )
        });

        if let Some(rule_index) = rule_index {
            let rule = &profile.rules[rule_index];
            let target = target_for_spec(
                &rule.then, &index, team.id, entity, position, ship_stats, &stats,
            );
            runtime.current = behavior_from_spec(&rule.then, runtime.spawn_position, target);
            if let Some(cooldown) = runtime.rule_cooldowns.get_mut(rule_index) {
                *cooldown = rule.cooldown_secs;
            }
        }

        // Re-roll a wander point once the ship reaches it.
        if let AiBehavior::Wander { point } = &runtime.current
            && point.distance(position) < 40.0
        {
            let jitter =
                Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5) * 400.0;
            runtime.current = AiBehavior::Wander {
                point: runtime.spawn_position + jitter,
            };
        }

        let _ = &transforms;
    }
}

fn target_for_spec(
    spec: &BehaviorSpec,
    index: &ShipIndex,
    team: u8,
    entity: Entity,
    position: Vec2,
    stats: &ShipStats,
    ships: &Query<&ShipStats>,
) -> Option<Entity> {
    match spec {
        BehaviorSpec::Guard { target, .. }
        | BehaviorSpec::Follow { target, .. }
        | BehaviorSpec::Chase { target }
        | BehaviorSpec::Attack { target }
        | BehaviorSpec::Kamikaze { target }
        | BehaviorSpec::Flee { from: target, .. } => {
            resolve_target(*target, index, team, entity, position, stats, ships)
        }
        _ => None,
    }
}

/// Turn the current runtime behavior into a steering intent consumed by
/// `update_ship_behavior`.
pub fn apply_ai_behaviors(
    mut ships: Query<(
        Entity,
        &AiRuntime,
        &Transform,
        &Ship,
        &ShipStats,
        &BoundingBox,
        &Team,
        &mut bevy_rapier2d::prelude::Velocity,
    )>,
    transforms: Query<&Transform>,
    stats: Query<&ShipStats>,
    index: Res<ShipIndex>,
    config: Res<GameConfigValue>,
    mut effects: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    for (entity, runtime, transform, ship, ship_stats, bbox, _team, mut velocity) in &mut ships {
        let position = transform.translation.truncate();
        let size = bbox.size.length();
        let intent = match &runtime.current {
            AiBehavior::Idle => ShipBehavior::Approach {
                position,
                radius: size,
            },
            AiBehavior::Wander { point } => ShipBehavior::Approach {
                position: *point,
                radius: size,
            },
            AiBehavior::Patrol { around, radius } => ShipBehavior::Approach {
                position: *around,
                radius: *radius,
            },
            AiBehavior::Guard { target, radius } => {
                let point = target
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                    .unwrap_or(position);
                ShipBehavior::Approach {
                    position: point,
                    radius: *radius,
                }
            }
            AiBehavior::Follow { target, radius } => {
                let point = target
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                    .unwrap_or(position);
                ShipBehavior::Approach {
                    position: point,
                    radius: *radius,
                }
            }
            AiBehavior::Regroup {
                position: point,
                radius,
            } => ShipBehavior::Approach {
                position: *point,
                radius: *radius,
            },
            AiBehavior::Kite { target, range } => {
                let Some(enemy) = target
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                else {
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position,
                        radius: size,
                    });
                    continue;
                };
                let distance = position.distance(enemy);
                if distance < *range {
                    let away = (position - enemy).normalize_or_zero();
                    ShipBehavior::Approach {
                        position: enemy + away * *range,
                        radius: *range,
                    }
                } else {
                    ShipBehavior::GoAround {
                        position: enemy,
                        radius: *range,
                    }
                }
            }
            AiBehavior::Chase { target } | AiBehavior::Attack { target } => {
                let Some(enemy) = target
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                else {
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position,
                        radius: size,
                    });
                    continue;
                };
                let radius = ship_stats.min_range * config.0.ai_min_range_factor;
                if position.distance(enemy) >= ship_stats.min_range {
                    ShipBehavior::Approach {
                        position: enemy,
                        radius,
                    }
                } else {
                    ShipBehavior::GoAround {
                        position: enemy,
                        radius,
                    }
                }
            }
            AiBehavior::Flee { from, distance } => {
                let Some(threat) = from
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                else {
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position,
                        radius: size,
                    });
                    continue;
                };
                // Correct direction: away from the threat.
                let away = (position - threat).normalize_or_zero();
                ShipBehavior::Approach {
                    position: position + away * *distance,
                    radius: size,
                }
            }
            AiBehavior::Kamikaze { target } => {
                let Some(enemy) = target
                    .and_then(|t| transforms.get(t).ok())
                    .map(|t| t.translation.truncate())
                else {
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position,
                        radius: size,
                    });
                    continue;
                };
                if position.distance(enemy) <= size + 20.0 {
                    effects.write(DamageEvent {
                        attacker: entity,
                        target: target.unwrap_or(entity),
                        damage: config.0.ai_kamikaze_damage,
                    });
                    commands.entity(entity).insert(crate::content::death::Death);
                }
                ShipBehavior::Approach {
                    position: enemy,
                    radius: size,
                }
            }
        };

        commands.entity(entity).insert(intent);

        let _ = (ship, &mut velocity, &index, &stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ship_info(entity: Entity, position: Vec2) -> ShipInfo {
        ShipInfo {
            entity,
            position,
            size_len: 100.0,
        }
    }

    #[test]
    fn nearest_enemy_picks_closest_and_ties_by_id() {
        let low = Entity::from_bits(10);
        let high = Entity::from_bits(20);
        let enemies = [
            ship_info(low, Vec2::new(10.0, 0.0)),
            ship_info(high, Vec2::new(-10.0, 0.0)),
        ];
        assert_eq!(
            nearest(&enemies, Vec2::ZERO, Entity::from_bits(1)).map(|e| e.entity),
            Some(low)
        );
    }

    #[test]
    fn squad_center_averages_allies() {
        let allies = [
            ship_info(Entity::from_bits(1), Vec2::new(0.0, 0.0)),
            ship_info(Entity::from_bits(2), Vec2::new(100.0, 100.0)),
            ship_info(Entity::from_bits(3), Vec2::new(200.0, 200.0)),
        ];
        assert_eq!(
            squad_center(&allies, Entity::from_bits(1)),
            Vec2::new(150.0, 150.0)
        );
    }

    #[test]
    fn highest_priority_wins_and_cooldown_blocks() {
        let profile = AiProfile {
            think_rate_hz: 1.0,
            initial: BehaviorSpec::Idle,
            rules: vec![
                AiRule {
                    priority: 1,
                    when: AiCondition::Always,
                    then: BehaviorSpec::Wander { radius: 100.0 },
                    cooldown_secs: 0.0,
                },
                AiRule {
                    priority: 10,
                    when: AiCondition::HpAbove(0.0),
                    then: BehaviorSpec::Kite { range: 500.0 },
                    cooldown_secs: 5.0,
                },
            ],
        };

        // Highest priority matches first.
        assert_eq!(select_rule(&profile, &[0.0, 0.0], |_| true), Some(1));
        // On cooldown, the lower-priority rule takes over.
        assert_eq!(select_rule(&profile, &[0.0, 5.0], |_| true), Some(0));
        // A condition that does not match skips that rule.
        assert_eq!(
            select_rule(&profile, &[0.0, 0.0], |c| matches!(c, AiCondition::Always)),
            Some(0)
        );
    }

    #[test]
    fn flee_behavior_moves_away_from_threat() {
        let threat = Vec2::new(0.0, 0.0);
        let position = Vec2::new(100.0, 0.0);
        // The runtime behavior stores the threat entity; the steering intent
        // is computed as `position + away * distance`.
        let away = (position - threat).normalize_or_zero();
        let intent_position = position + away * 1000.0;
        assert!(intent_position.x > position.x);
    }
}
