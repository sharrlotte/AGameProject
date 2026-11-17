use bevy::{math::FloatPow, prelude::*};
use std::f32::consts::PI;

use super::{Turret, TurretBarrel, TurretSprite, TurretState, TurretTarget};
use crate::{
    content::{
        block::{component::TargetingStrategy, radar::Radar},
        bullet::{Interceptable, ProjectileOwner},
        ship::{Ship, ShipGrid, ShipStats},
        team::Team,
    },
    resource::SceneState,
    simulation::SimulationSet,
};
use bevy_rapier2d::prelude::Velocity;

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (turret_tracking, update_turret_target)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Weapons),
        );
    }
}

fn target_entity(target: TurretTarget) -> Entity {
    match target {
        TurretTarget::Ship(entity) | TurretTarget::Projectile(entity) => entity,
    }
}

/// Solve the intercept point for a moving target and a finite projectile
/// speed. Falls back to the target's current position when there is no
/// positive-time solution (e.g. instantaneous beams).
pub fn lead_point(origin: Vec2, target: Vec2, velocity: Vec2, projectile_speed: f32) -> Vec2 {
    if !projectile_speed.is_finite() || projectile_speed <= 0.0 {
        return target;
    }

    let delta = target - origin;
    let quadratic_a = velocity.length_squared() - projectile_speed * projectile_speed;
    let quadratic_b = 2.0 * delta.dot(velocity);
    let quadratic_c = delta.length_squared();

    let time = if quadratic_a.abs() < 1e-6 {
        if quadratic_b.abs() < 1e-6 {
            0.0
        } else {
            -quadratic_c / quadratic_b
        }
    } else {
        let discriminant = quadratic_b * quadratic_b - 4.0 * quadratic_a * quadratic_c;
        if discriminant < 0.0 {
            0.0
        } else {
            let root = discriminant.sqrt();
            let t1 = (-quadratic_b + root) / (2.0 * quadratic_a);
            let t2 = (-quadratic_b - root) / (2.0 * quadratic_a);
            [t1, t2]
                .into_iter()
                .filter(|t| t.is_finite() && *t > 0.0)
                .fold(f32::INFINITY, f32::min)
        }
    };

    if time.is_finite() && time > 0.0 {
        target + velocity * time
    } else {
        target
    }
}

/// True when `entity` still has a live core block, i.e. it is a valid ship
/// target. A core-less wreck must never be targeted.
fn has_core(grid: &ShipGrid, ship: &Ship) -> bool {
    grid.block(ship.id).is_some()
}

#[derive(Clone, Copy)]
struct Candidate {
    target: TurretTarget,
    aim: Vec2,
    dist_sq: f32,
    health: f32,
    priority: f32,
    id: u64,
}

/// Closest wins on distance; ties break by entity id.
fn better_closest(candidate: Candidate, best: Option<Candidate>) -> bool {
    match best {
        None => true,
        Some(best) => {
            candidate.dist_sq < best.dist_sq
                || ((candidate.dist_sq - best.dist_sq).abs() < 1e-6 && candidate.id < best.id)
        }
    }
}

/// MostHP wins on current health; ties break by distance, then entity id.
fn better_most_hp(candidate: Candidate, best: Option<Candidate>) -> bool {
    match best {
        None => true,
        Some(best) => {
            candidate.health > best.health
                || ((candidate.health - best.health).abs() < 1e-6
                    && (candidate.dist_sq < best.dist_sq
                        || ((candidate.dist_sq - best.dist_sq).abs() < 1e-6
                            && candidate.id < best.id)))
        }
    }
}

/// Point defense prioritises nearby missiles, then distance, then entity id.
fn better_intercept(candidate: Candidate, best: Option<Candidate>) -> bool {
    match best {
        None => true,
        Some(best) => {
            candidate.priority > best.priority
                || ((candidate.priority - best.priority).abs() < 1e-6
                    && (candidate.dist_sq < best.dist_sq
                        || ((candidate.dist_sq - best.dist_sq).abs() < 1e-6
                            && candidate.id < best.id)))
        }
    }
}

pub fn update_turret_target(
    ships: Query<(&GlobalTransform, &Ship, &ShipGrid, &ShipStats)>,
    projectiles: Query<
        (
            Entity,
            &GlobalTransform,
            &Velocity,
            &ProjectileOwner,
            Option<&crate::content::bullet::Missile>,
        ),
        With<Interceptable>,
    >,
    mut turrets: Query<(&Turret, &mut TurretState, &Team, &Radar, &TurretBarrel)>,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
) {
    for (turret, mut state, team, radar, barrel_ref) in turrets.iter_mut() {
        let Ok(transform) = barrels.get(barrel_ref.0) else {
            continue;
        };
        let turret_position = transform.translation().truncate();
        let range_sq = turret.range.squared();
        let mut best: Option<Candidate> = None;

        for entity in radar.targets.iter().copied() {
            let Ok((target_transform, ship, grid, stats)) = ships.get(entity) else {
                continue;
            };
            if !has_core(grid, ship) {
                continue;
            }
            let aim = target_transform
                .transform_point(ship.core_position.extend(0.0))
                .truncate();
            let dist_sq = turret_position.distance_squared(aim);
            if dist_sq > range_sq {
                continue;
            }

            let candidate = Candidate {
                target: TurretTarget::Ship(entity),
                aim,
                dist_sq,
                health: stats.health,
                priority: 0.0,
                id: entity.to_bits(),
            };
            let is_better = match turret.strategy {
                TargetingStrategy::Closest => better_closest(candidate, best),
                TargetingStrategy::MostHP => better_most_hp(candidate, best),
                TargetingStrategy::EnemyBullet => false,
            };
            if is_better {
                best = Some(candidate);
            }
        }

        if matches!(turret.strategy, TargetingStrategy::EnemyBullet) {
            for (entity, projectile_transform, velocity, owner, missile) in &projectiles {
                if owner.team == team.id {
                    continue;
                }
                let position = projectile_transform.translation().truncate();
                let dist_sq = turret_position.distance_squared(position);
                if dist_sq > range_sq {
                    continue;
                }
                let priority = if missile.is_some() && dist_sq <= range_sq * 0.25 {
                    1.0
                } else {
                    0.0
                };
                let candidate = Candidate {
                    target: TurretTarget::Projectile(entity),
                    aim: lead_point(
                        turret_position,
                        position,
                        velocity.linear,
                        turret.shooting_pattern.projectile_speed(),
                    ),
                    dist_sq,
                    health: 0.0,
                    priority,
                    id: entity.to_bits(),
                };
                if better_intercept(candidate, best) {
                    best = Some(candidate);
                }
            }
        }

        if let Some(candidate) = best {
            let to_target = candidate.aim - turret_position;
            let target_dir = to_target.normalize_or_zero();
            let forward: Vec2 = transform.up().truncate();
            let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));

            state.target = Some(candidate.target);
            state.aim_point = candidate.aim;
            state.can_fire = angle_error.abs() < PI / 16.0;
            state.active = state.can_fire;
        } else {
            state.target = None;
            state.aim_point = turret_position;
            state.can_fire = false;
            state.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: u64, dist_sq: f32, health: f32) -> Candidate {
        Candidate {
            target: TurretTarget::Ship(Entity::from_bits(id)),
            aim: Vec2::ZERO,
            dist_sq,
            health,
            priority: 0.0,
            id,
        }
    }

    #[test]
    fn closest_ties_break_by_entity_id() {
        let first = candidate(1, 100.0, 10.0);
        let second = candidate(2, 100.0, 999.0);
        // Equal distance: the lower entity id wins.
        assert!(!better_closest(second, Some(first)));
        assert!(better_closest(first, Some(second)));
    }

    #[test]
    fn most_hp_uses_health_then_distance_then_id() {
        let low_hp_close = candidate(1, 1.0, 10.0);
        let high_hp_far = candidate(2, 400.0, 100.0);
        assert!(better_most_hp(high_hp_far, Some(low_hp_close)));
        assert!(!better_most_hp(low_hp_close, Some(high_hp_far)));

        let equal_a = candidate(1, 100.0, 100.0);
        let equal_b = candidate(2, 100.0, 100.0);
        assert!(better_most_hp(equal_b, Some(equal_a)) == false);
    }

    #[test]
    fn point_defense_prioritises_close_missiles() {
        let mut bullet = candidate(1, 10.0, 0.0);
        let mut missile = candidate(2, 20.0, 0.0);
        missile.priority = 1.0;
        assert!(better_intercept(missile, Some(bullet)));
        assert!(!better_intercept(bullet, Some(missile)));

        bullet.dist_sq = 1.0;
        missile.priority = 0.0;
        assert!(better_intercept(bullet, Some(missile)));
    }

    #[test]
    fn lead_point_aheads_a_moving_target() {
        let aim = lead_point(
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(50.0, 0.0),
            100.0,
        );
        assert!(aim.x > 0.0, "aim should lead the target: {aim:?}");
        assert!((aim.y - 100.0).abs() < 1e-3);

        // Instantaneous beams aim directly at the target.
        let instant = lead_point(
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(50.0, 0.0),
            f32::INFINITY,
        );
        assert_eq!(instant, Vec2::new(0.0, 100.0));
    }
}

pub fn turret_tracking(
    mut barrel_transforms: Query<&mut Transform, With<TurretSprite>>,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    turrets: Query<(&Turret, &TurretState, &TurretBarrel)>,
    time: Res<Time>,
    transforms: Query<(&Ship, &GlobalTransform)>,
) {
    for (turret, state, barrel_ref) in &turrets {
        let turn_speed = turret.turn_speed;

        if turn_speed == 0. {
            continue;
        }

        let Ok(global_transform) = barrels.get(barrel_ref.0) else {
            continue;
        };

        let mut target_position: Option<Vec2> = None;

        if let Some(TurretTarget::Ship(target)) = state.target {
            if let Ok((ship, position)) = transforms.get(target) {
                target_position = Some(
                    position
                        .transform_point(ship.core_position.extend(0.0))
                        .truncate(),
                );
            }
        }

        if let Some(shoot_at_position) = target_position {
            let current_position = global_transform.translation().truncate();
            let diff = shoot_at_position - current_position;
            let dist_sq = diff.length_squared();

            if dist_sq < 0.0001 {
                continue;
            }

            let forward: Vec2 = global_transform.up().truncate();

            let to_target = shoot_at_position - global_transform.translation().truncate();

            if to_target.length_squared() < 0.0001 {
                continue;
            }

            let target_dir = to_target.normalize_or_zero();
            let angle_error: f32 = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
            let max_turn = turn_speed * time.delta_secs();
            let turn = angle_error.clamp(-max_turn, max_turn);

            if let Ok(mut transform) = barrel_transforms.get_mut(barrel_ref.0) {
                transform.rotate_z(turn);
            }
        }
    }
}
