use crate::{content::team::Team, turret::control::FireIntent};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TurretTargeter {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub target: Option<Entity>,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

impl Default for TurretTargeter {
    fn default() -> Self {
        Self {
            strategy: TargetingStrategy::Closest,
            range: 500.0,
            target: None,
        }
    }
}

pub fn update_turret_target(
    mut turret_query: Query<(&GlobalTransform, &mut TurretTargeter, &Team)>,
    // Assuming potential targets have GlobalTransform and Team.
    // In a real game, you might filter by a component like `Hittable` or `Ship`.
    target_query: Query<(Entity, &GlobalTransform, &Team)>,
) {
    for (turret_transform, mut targeter, turret_team) in turret_query.iter_mut() {
        let turret_pos = turret_transform.translation().truncate();
        let mut best_target: Option<Entity> = None;

        // Initialize with range squared to filter out targets that are too far
        let mut closest_dist_sq = targeter.range * targeter.range;

        // Iterate all potential targets
        for (target_entity, target_transform, target_team) in target_query.iter() {
            if turret_team.id == target_team.id {
                continue; // Skip allies
            }

            let target_pos = target_transform.translation().truncate();
            let dist_sq = turret_pos.distance_squared(target_pos);

            // Optimization: Skip if out of range immediately
            if dist_sq > targeter.range * targeter.range {
                continue;
            }

            match targeter.strategy {
                TargetingStrategy::Closest => {
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        best_target = Some(target_entity);
                    }
                }
                TargetingStrategy::MostHP => {
                    // Placeholder: Fallback to closest since we don't have HP component yet
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        best_target = Some(target_entity);
                    }
                }
                TargetingStrategy::EnemyBullet => {
                    // Placeholder: Fallback to closest
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        best_target = Some(target_entity);
                    }
                }
            }
        }

        targeter.target = best_target;
    }
}

pub fn apply_targeting_to_fire_intent(
    mut query: Query<(&mut FireIntent, &TurretTargeter)>,
    target_query: Query<&GlobalTransform>,
) {
    for (mut intent, targeter) in query.iter_mut() {
        if let Some(target_entity) = targeter.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                intent.target_pos = target_transform.translation().truncate();
                intent.active = true;
            } else {
                intent.active = false;
            }
        } else {
            intent.active = false;
        }
    }
}
