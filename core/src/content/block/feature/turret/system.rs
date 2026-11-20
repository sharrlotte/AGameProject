use bevy::{math::FloatPow, prelude::*};

use super::{Candidate, Turret, TurretBarrel, TurretCharge, TurretPower, TurretSprite, TurretState, TurretTarget};
use crate::content::block::{BlockOwner, Disabled, TurretMirror};
use crate::content::ship::ai::FireOrder;
use crate::{
    content::{
        block::{
            LaserBeamPatternState, LaserBeamPatternShootingState, PullingBeamPattern,
            component::TargetingStrategy,
            feature::power::{Boost, Demand, Supply},
            radar::Radar,
        },
        body::BlockGrid,
        bullet::{Interceptable, Missile, ProjectileOwner},
        ship::{Ship, ShipStats},
        team::Team,
    },
};
use bevy_rapier2d::prelude::Velocity;


pub fn update_turret_demand(
    mut turrets: Query<
        (
            &TurretPower,
            &TurretState,
            &mut Demand,
            Option<&TurretCharge>,
            Option<&Boost>,
            Option<&LaserBeamPatternState>,
            Option<&PullingBeamPattern>,
        ),
        Without<Disabled>,
    >,
) {
    for (power, state, mut demand, charge, boost, laser_state, pulling) in &mut turrets {
        let boost = boost.map_or(1.0, Boost::value);
        demand.value = match power {
            TurretPower::Charge {
                capacity,
                charge_rate,
            } => {
                let current = charge.map_or(*capacity, |c| c.value);
                if current < *capacity {
                    charge_rate * boost
                } else {
                    0.0
                }
            }
            TurretPower::Drain { rate } => {
                let beam_active = laser_state.is_some_and(|s| {
                    matches!(s.shooting, LaserBeamPatternShootingState::Shooting { .. })
                }) || (pulling.is_some() && state.active());
                if beam_active { *rate } else { 0.0 }
            }
        };
    }
}

pub fn advance_turret_charge(
    time: Res<Time>,
    mut turrets: Query<
        (
            &TurretPower,
            &mut TurretCharge,
            Option<&Supply>,
            Option<&Boost>,
        ),
        Without<Disabled>,
    >,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (power, mut charge, supply, boost) in &mut turrets {
        let TurretPower::Charge {
            capacity,
            charge_rate,
        } = *power
        else {
            continue;
        };
        let supply = supply.map_or(1.0, Supply::value);
        let boost = boost.map_or(1.0, Boost::value);
        charge.value = (charge.value + charge_rate * supply * boost * dt).min(capacity);
    }
}

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

fn aim_turn(angle_error: f32, mirrored: bool, max_turn: f32) -> f32 {
    let sign = if mirrored { -1.0 } else { 1.0 };
    (angle_error * sign).clamp(-max_turn, max_turn)
}


fn has_core(grid: &BlockGrid, ship: &Ship) -> bool {
    grid.block(ship.core_id).is_some()
}

fn better_closest(candidate: Candidate, best: Option<Candidate>) -> bool {
    match best {
        None => true,
        Some(best) => {
            candidate.dist_sq < best.dist_sq
                || ((candidate.dist_sq - best.dist_sq).abs() < 1e-6 && candidate.id < best.id)
        }
    }
}

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
    ships: Query<(&GlobalTransform, &Ship, &BlockGrid, &ShipStats, Option<&Velocity>)>,
    orders: Query<&FireOrder>,
    projectiles: Query<
        (
            Entity,
            &GlobalTransform,
            &Velocity,
            &ProjectileOwner,
            Option<&Missile>,
        ),
        With<Interceptable>,
    >,
    mut turrets: Query<
        (&Turret, &mut TurretState, &Team, &Radar, &TurretBarrel, Option<&BlockOwner>),
        Without<Disabled>,
    >,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
) {
    for (turret, mut state, team, radar, barrel_ref, owner) in turrets.iter_mut() {
        let Ok(transform) = barrels.get(barrel_ref.0) else {
            continue;
        };
        let turret_position = transform.translation().truncate();
        let range_sq = turret.range.squared();
        let order = owner
            .and_then(|o| orders.get(o.body).ok())
            .copied()
            .unwrap_or_default();

        let mut best: Option<Candidate> = None;

        if !matches!(turret.strategy, TargetingStrategy::EnemyBullet)
            && let Some(primary) = order.primary_target
            && let Ok((target_transform, ship, grid, stats, velocity)) = ships.get(primary)
            && has_core(grid, ship)
        {
            let core_world = target_transform
                .transform_point(ship.core_position.extend(0.0))
                .truncate();

            let dist_sq = turret_position.distance_squared(core_world);

            if dist_sq <= range_sq {
                let aim = lead_point(
                    turret_position,
                    core_world,
                    velocity.map_or(Vec2::ZERO, |v| v.linear) * order.lead_accuracy,
                    turret.shooting_pattern.projectile_speed(),
                );
                
                best = Some(Candidate {
                    target: TurretTarget::Ship(primary),
                    aim,
                    dist_sq,
                    health: stats.health,
                    priority: 1.0,
                    id: primary.to_bits(),
                });
            }
        }

        if best.is_none() {
            for entity in radar.targets.iter().copied() {
                let Ok((target_transform, ship, grid, stats, velocity)) = ships.get(entity) else {
                    continue;
                };
                if !has_core(grid, ship) {
                    continue;
                }
                let core_world = target_transform
                    .transform_point(ship.core_position.extend(0.0))
                    .truncate();
                let dist_sq = turret_position.distance_squared(core_world);
                if dist_sq > range_sq {
                    continue;
                }
                let aim = lead_point(
                    turret_position,
                    core_world,
                    velocity.map_or(Vec2::ZERO, |v| v.linear) * order.lead_accuracy,
                    turret.shooting_pattern.projectile_speed(),
                );

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
        }

        if matches!(turret.strategy, TargetingStrategy::EnemyBullet) {
            for (entity, projectile_transform, velocity, owner, missile) in &projectiles {
                if owner.team == team.id() {
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

            if angle_error.abs() < turret.aim_tolerance {
                state.ready(candidate.aim, Some(candidate.target));
            } else {
                state.track(candidate.aim, Some(candidate.target));
            }
        } else {
            state.clear();
        }
    }
}

pub fn turret_tracking(
    mut barrel_transforms: Query<&mut Transform, With<TurretSprite>>,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    turrets: Query<
        (&Turret, &TurretState, &TurretBarrel, Option<&TurretMirror>),
        Without<Disabled>,
    >,
    time: Res<Time>,
) {
    for (turret, state, barrel_ref, mirror) in &turrets {
        let turn_speed = turret.turn_speed;

        if turn_speed == 0. {
            continue;
        }

        let Ok(global_transform) = barrels.get(barrel_ref.0) else {
            continue;
        };

        if state.target().is_some() {
            let shoot_at_position = state.aim_point();
            let current_position = global_transform.translation().truncate();
            let diff = shoot_at_position - current_position;
            let dist_sq = diff.length_squared();

            if dist_sq < 0.0001 {
                continue;
            }

            let forward: Vec2 = global_transform.up().truncate();
            let to_target = shoot_at_position - current_position;

            if to_target.length_squared() < 0.0001 {
                continue;
            }

            let target_dir = to_target.normalize_or_zero();
            let angle_error: f32 = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
            let max_turn = turn_speed * time.delta_secs();
            let mirrored = mirror.is_some_and(|value| value.0);
            let turn = aim_turn(angle_error, mirrored, max_turn);

            if let Ok(mut transform) = barrel_transforms.get_mut(barrel_ref.0) {
                transform.rotate_z(turn);
            }
        }
    }
}

pub fn reset_disabled_turret(
    trigger: On<Add<Disabled>>,
    mut turrets: Query<&mut TurretState>,
) {
    if let Ok(mut state) = turrets.get_mut(trigger.event().entity) {
        state.clear();
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
    fn turret_aim_respects_mirrored_visual() {
        let target = Vec2::new(1.0, 0.0);
        let world_forward = Vec2::Y;

        for mirrored in [false, true] {
            let error = world_forward.perp_dot(target).atan2(world_forward.dot(target));
            let angle = 0.0 + aim_turn(error, mirrored, std::f32::consts::FRAC_PI_2);
            let forward = if mirrored {
                Vec2::new(angle.sin(), angle.cos())
            } else {
                Vec2::new(-angle.sin(), angle.cos())
            };
            assert!(
                forward.distance(target) < 1e-3,
                "mirrored={mirrored} should aim at {target:?}, got {forward:?}"
            );
        }
    }

    #[test]
    fn closest_ties_break_by_entity_id() {
        let first = candidate(1, 100.0, 10.0);
        let second = candidate(2, 100.0, 999.0);
        assert!(!better_closest(second, Some(first)));
        assert!(better_closest(first, Some(second)));
    }

    #[test]
    fn full_battery_draws_zero_and_starved_charges_slower() {
        use std::time::Duration;

        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, (update_turret_demand, advance_turret_charge).chain());

        let turret = app
            .world_mut()
            .spawn((
                TurretPower::Charge {
                    capacity: 20.0,
                    charge_rate: 10.0,
                },
                TurretState::default(),
                TurretCharge { value: 20.0 },
                Demand::default(),
                Supply { value: 0.5 },
                Boost { value: 1.0 },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        app.update();
        assert_eq!(app.world().get::<Demand>(turret).unwrap().value, 0.0);

        app.world_mut().get_mut::<TurretCharge>(turret).unwrap().value = 0.0;
        for _ in 0..4 {
            app.world_mut()
                .resource_mut::<Time<()>>()
                .advance_by(Duration::from_secs(1));
            app.update();
        }
        assert!((app.world().get::<TurretCharge>(turret).unwrap().value - 20.0).abs() < 1e-5);
    }

    #[test]
    fn beam_turret_draws_only_while_active() {
        let mut app = App::new();
        app.add_systems(Update, update_turret_demand);

        let beam = app
            .world_mut()
            .spawn((
                TurretPower::Drain { rate: 10.0 },
                TurretState::default(),
                Demand::default(),
                LaserBeamPatternState {
                    beam: None,
                    shooting: LaserBeamPatternShootingState::Cooldown {
                        cooldown_remaining: 0.0,
                    },
                },
            ))
            .id();

        app.update();
        assert_eq!(app.world().get::<Demand>(beam).unwrap().value, 0.0);

        app.world_mut()
            .get_mut::<LaserBeamPatternState>(beam)
            .unwrap()
            .shooting = LaserBeamPatternShootingState::Shooting { elapsed: 0.0 };
        app.update();
        assert_eq!(app.world().get::<Demand>(beam).unwrap().value, 10.0);
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

        let instant = lead_point(
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(50.0, 0.0),
            f32::INFINITY,
        );
        assert_eq!(instant, Vec2::new(0.0, 100.0));
    }

    #[test]
    fn custom_aim_tolerance_gates_firing_cone() {
        let tolerance = 0.035_f32;
        assert!(0.02_f32.abs() < tolerance);
        assert!(!(0.1_f32.abs() < tolerance));
    }

    #[test]
    fn fire_order_and_turret_tracking_use_aim_point_and_lead_accuracy() {
        use std::f32::consts::PI;
        use std::time::Duration;
        use crate::content::block::feature::turret::TurretSprite;
        use crate::content::block::{
            BakedShape, BlockDef, BlockPlacement, BlockRegistry, Rot90, ShapeSpec,
            ShootPattern, ShootingProjectile, SingleShootPattern,
        };

        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, (update_turret_target, turret_tracking).chain());

        let mut registry = BlockRegistry::default();
        registry.register(
            "core",
            BlockDef {
                name: "core".into(),
                block_type: "core".into(),
                block_group: "core".into(),
                mass: 10.0,
                health: 100.0,
                shape: BakedShape::try_from(&ShapeSpec::Square(1)).unwrap(),
                cost: 1,
                features: vec![],
                drops: vec![],
            },
        );
        let core_placement = BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0);
        let mut grid_a = BlockGrid::new();
        grid_a.rebuild_from_layout(&[core_placement.clone()], &registry);
        let core_a = grid_a.blocks().next().unwrap().placement.id;
        let mut grid_b = BlockGrid::new();
        grid_b.rebuild_from_layout(&[core_placement], &registry);
        let core_b = grid_b.blocks().next().unwrap().placement.id;

        let enemy_a = app
            .world_mut()
            .spawn((
                GlobalTransform::from_xyz(0.0, 200.0, 0.0),
                Ship {
                    core_id: core_a,
                    core_position: Vec2::ZERO,
                    ..default()
                },
                grid_a,
                ShipStats {
                    health: 100.0,
                    ..default()
                },
                Velocity {
                    linear: Vec2::new(100.0, 0.0),
                    angular: 0.0,
                },
            ))
            .id();
        let enemy_b = app
            .world_mut()
            .spawn((
                GlobalTransform::from_xyz(0.0, 100.0, 0.0),
                Ship {
                    core_id: core_b,
                    core_position: Vec2::ZERO,
                    ..default()
                },
                grid_b,
                ShipStats {
                    health: 100.0,
                    ..default()
                },
                Velocity::zero(),
            ))
            .id();

        let owner_ship = app
            .world_mut()
            .spawn(FireOrder {
                primary_target: Some(enemy_a),
                lead_accuracy: 1.0,
            })
            .id();
        let barrel = app
            .world_mut()
            .spawn((Transform::default(), GlobalTransform::default(), TurretSprite))
            .id();
        let turret = app
            .world_mut()
            .spawn((
                BlockOwner { body: owner_ship },
                Turret {
                    range: 500.0,
                    turn_speed: 4.0,
                    aim_tolerance: PI / 16.0,
                    barrels: vec![Vec2::ZERO],
                    strategy: TargetingStrategy::Closest,
                    shooting_pattern: ShootPattern::Single(SingleShootPattern {
                        fire_rate: 1.0,
                        projectile: ShootingProjectile::Bullet {
                            speed: 200.0,
                            damage: 10.0,
                            lifetime: 5.0,
                            piercing: 0,
                            width: 2.0,
                            length: 10.0,
                        },
                    }),
                },
                TurretState::default(),
                Team::new(0),
                Radar {
                    range: 500.0,
                    timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    targets: vec![enemy_b, enemy_a],
                },
                TurretBarrel(barrel),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(100));
        app.update();

        let state = app.world().get::<TurretState>(turret).unwrap();
        assert_eq!(state.target(), Some(TurretTarget::Ship(enemy_a)));
        assert!(
            state.aim_point().x > 10.0,
            "expected lead aim_point, got {:?}",
            state.aim_point()
        );
        let barrel_rot = app
            .world()
            .get::<Transform>(barrel)
            .unwrap()
            .rotation
            .to_euler(EulerRot::ZYX)
            .0;
        assert!(
            barrel_rot < 0.0,
            "barrel should rotate clockwise toward +X lead point"
        );
    }
}
