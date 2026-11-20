use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;

use super::component::{
    Brain, Capability, FireOrder, Posture, Readiness, SharedContact, Skill, Steering,
    SupportKind, TacticalBoard, Tactic, TeamPlan,
};
use super::config::{AiConfig, AiConfigValue};
use crate::config::GameConfigValue;
use crate::content::block::feature::turret::lead_point;
use crate::content::ship::{Ship, ShipStats};
use crate::content::team::Team;
use crate::resource::ControlledShip;

#[cfg(feature = "dev")]
use crate::content::ship::{BoundingBox, ship_center};

#[cfg(feature = "dev")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteeringGizmo {
    pub origin: Vec2,
    pub desired_heading: Option<f32>,
    pub thrust_world: Vec2,
}

#[cfg(feature = "dev")]
pub fn steering_gizmo_vectors(
    steering: &Steering,
    transform: &Transform,
    bounds: &BoundingBox,
) -> SteeringGizmo {
    SteeringGizmo {
        origin: ship_center(transform, bounds),
        desired_heading: steering.desired_heading,
        thrust_world: steering.thrust_world,
    }
}

pub fn desired_ship_angle(direction: Vec2, mount_offset_angle: f32) -> f32 {
    direction.y.atan2(direction.x) - FRAC_PI_2 - mount_offset_angle
}

pub fn steer_ships(
    time: Res<Time>,
    ai_cfg: Res<AiConfigValue>,
    game_cfg: Res<GameConfigValue>,
    board: Res<TacticalBoard>,
    controlled: Res<ControlledShip>,
    ship_transforms: Query<(&GlobalTransform, &Ship)>,
    mut ships: Query<(
        Entity,
        &Brain,
        &Skill,
        &Capability,
        &Readiness,
        &Team,
        &Ship,
        &ShipStats,
        &Transform,
        &Velocity,
        &mut Steering,
        &mut FireOrder,
    )>,
) {
    let elapsed = time.elapsed_secs();
    let empty_plan = TeamPlan::default();

    for (
        entity,
        brain,
        skill,
        cap,
        readiness,
        team,
        ship,
        stats,
        transform,
        velocity,
        mut steering,
        mut fire_order,
    ) in &mut ships
    {
        let level = skill.level();
        let lead_accuracy = ai_cfg.0.curves.lead_accuracy.sample(level);

        if controlled.controlling == Some(entity) {
            *fire_order = FireOrder {
                primary_target: None,
                lead_accuracy: 1.0,
            };
            continue;
        }

        let pos = transform
            .transform_point(ship.core_position.extend(0.0))
            .truncate();
        let plan = board.teams.get(&team.id()).unwrap_or(&empty_plan);
        let target_entity = brain.current.target_entity();
        let contact = target_entity.and_then(|e| plan.intel.contacts.get(&e).copied());

        let (next_steer, primary_target) = match brain.current {
            Tactic::Engage {
                target,
                range,
                posture,
            } => steer_engage(
                entity,
                pos,
                velocity.linear,
                brain,
                level,
                cap,
                readiness,
                &ai_cfg.0,
                game_cfg.0.linear_damping,
                elapsed,
                target,
                range,
                posture,
                contact,
                &ship_transforms,
            ),
            Tactic::Recharge { threat, safe_range } => (
                steer_away_from(
                    pos,
                    threat,
                    Some(safe_range),
                    ai_cfg.0.combat_tuning.recharge_throttle_cap,
                    contact,
                    &ship_transforms,
                ),
                Some(threat),
            ),
            Tactic::Retreat { threat } => (
                steer_away_from(pos, threat, None, 1.0, contact, &ship_transforms),
                None,
            ),
            Tactic::Support { ally, kind } => {
                let steer = steer_support(
                    pos,
                    velocity.linear,
                    ally,
                    kind,
                    cap,
                    &ai_cfg.0,
                    stats.thrust_power / stats.mass.max(1.0),
                    game_cfg.0.linear_damping,
                    plan,
                    &ship_transforms,
                );
                (steer, plan.focus_target)
            }
            Tactic::Regroup { position }
            | Tactic::Investigate { position }
            | Tactic::Search { waypoint: position } => (
                steer_arrive(
                    pos,
                    velocity.linear,
                    position,
                    ai_cfg.0.investigate_arrival_radius,
                    stats.thrust_power / stats.mass.max(1.0),
                    game_cfg.0.linear_damping,
                ),
                None,
            ),
        };

        *steering = next_steer;
        *fire_order = FireOrder {
            primary_target,
            lead_accuracy,
        };
    }
}

#[allow(clippy::too_many_arguments)]
fn steer_engage(
    entity: Entity,
    pos: Vec2,
    vel: Vec2,
    brain: &Brain,
    skill: u8,
    cap: &Capability,
    readiness: &Readiness,
    cfg: &AiConfig,
    damping: f32,
    elapsed: f32,
    target: Entity,
    range: f32,
    posture: Posture,
    contact: Option<SharedContact>,
    ship_transforms: &Query<(&GlobalTransform, &Ship)>,
) -> (Steering, Option<Entity>) {
    let Some((target_pos, target_vel, has_fixed_gun, enemy_heading)) =
        resolve_contact_kinematics(target, contact, ship_transforms)
    else {
        return (Steering::default(), None);
    };

    let to_target = target_pos - pos;
    let dist = to_target.length();
    let dir = to_target.normalize_or_zero();
    let lead_accuracy = cfg.curves.lead_accuracy.sample(skill);
    let evasion = cfg.curves.evasion_strength.sample(skill);

    let ct = &cfg.combat_tuning;
    let mut desired_heading = None;
    let mut thrust_world = Vec2::ZERO;

    match posture {
        Posture::SpinalAim => {
            if let Some(fixed) = cap.fixed_mount {
                let mut aim = lead_point(
                    pos,
                    target_pos,
                    target_vel * lead_accuracy,
                    fixed.projectile_speed,
                );
                let jitter = cfg.curves.aim_jitter_rad.sample(skill);
                if jitter > 0.0 {
                    let wobble = (elapsed * 1.9 + (entity.to_bits() % 1024) as f32).sin() * jitter;
                    let off = aim - pos;
                    aim = pos + Vec2::from_angle(wobble).rotate(off);
                }
                let aim_dir = (aim - pos).normalize_or_zero();
                desired_heading = Some(desired_ship_angle(aim_dir, fixed.mount_angle));
            }
            if dist > range * 1.08 {
                thrust_world += dir * ((dist - range) / (range * 0.35)).clamp(0.25, 1.0);
            } else if dist < range * 0.85 {
                thrust_world -= dir * ((range - dist) / (range * 0.35)).clamp(0.2, 0.9);
            }
            if skill >= cfg.unlocks.strafe_evade_level {
                thrust_world += dir.perp() * brain.strafe_sign * (0.45 * evasion);
            }
        }
        Posture::Orbit | Posture::Kite | Posture::Brawl => {
            let radial_err = (dist - range) / (range * 0.35).max(100.0);
            let radial = dir * radial_err.clamp(-1.0, 1.0);
            let tangent_weight = if matches!(posture, Posture::Brawl) {
                ct.brawl_tangent_weight
            } else {
                ct.orbit_tangent_weight
            };
            let tangent = dir.perp() * brain.strafe_sign * tangent_weight;
            thrust_world = radial + tangent;

            if skill >= cfg.unlocks.armor_angling_level
                && let Some(angle) = best_armor_heading(cap, dir, ct.armor_angle_hp_ratio)
            {
                desired_heading = Some(angle);
            }
        }
    }

    if skill >= cfg.unlocks.spinal_dodge_level
        && has_fixed_gun
        && enemy_heading.dot(-dir) > ct.spinal_dodge_cos_threshold
    {
        thrust_world += dir.perp() * brain.strafe_sign * ct.spinal_dodge_impulse;
    }

    // Damp excess radial speed near target range
    let approach_speed = vel.dot(dir);
    let stop_dist = (approach_speed * approach_speed) / (2.0 * damping.max(0.1));
    if (dist - range).abs() < stop_dist * 0.4 && thrust_world.dot(dir) > 0.0 && approach_speed > 0.0
    {
        thrust_world -= dir * thrust_world.dot(dir);
    }

    let mut throttle_cap = 1.0;
    if skill >= cfg.unlocks.power_management_level
        && readiness.firing_drain_beam
        && readiness.power_supply < ct.drain_beam_supply_threshold
        && (dist - range).abs() < range * 0.25
    {
        throttle_cap = ct.drain_beam_throttle_cap;
    }

    (
        Steering {
            desired_heading,
            thrust_world: clamp_unit(thrust_world),
            throttle_cap,
        },
        Some(target),
    )
}

fn steer_away_from(
    pos: Vec2,
    threat: Entity,
    safe_range: Option<f32>,
    throttle_cap: f32,
    contact: Option<SharedContact>,
    ship_transforms: &Query<(&GlobalTransform, &Ship)>,
) -> Steering {
    let Some((threat_pos, _, _, _)) = resolve_contact_kinematics(threat, contact, ship_transforms)
    else {
        return Steering::default();
    };
    let away = (pos - threat_pos).normalize_or_zero();
    let dist = pos.distance(threat_pos);
    let thrust = match safe_range {
        Some(safe) if dist >= safe => away.perp() * 0.5,
        _ => away,
    };
    Steering {
        desired_heading: None,
        thrust_world: clamp_unit(thrust),
        throttle_cap,
    }
}

#[allow(clippy::too_many_arguments)]
fn steer_support(
    pos: Vec2,
    vel: Vec2,
    ally: Entity,
    kind: SupportKind,
    cap: &Capability,
    cfg: &AiConfig,
    decel: f32,
    damping: f32,
    plan: &TeamPlan,
    ship_transforms: &Query<(&GlobalTransform, &Ship)>,
) -> Steering {
    let Ok((gt, ship)) = ship_transforms.get(ally) else {
        return Steering::default();
    };
    let vip_pos = gt.transform_point(ship.core_position.extend(0.0)).truncate();
    let anchor = match kind {
        SupportKind::Overdrive => vip_pos,
        SupportKind::Screen => {
            if let Some(threat) = plan
                .focus_target
                .and_then(|e| plan.intel.contacts.get(&e))
                .or_else(|| plan.intel.contacts.values().next())
            {
                let to_threat = (threat.position - vip_pos).normalize_or_zero();
                vip_pos + to_threat * cfg.combat_tuning.support_screen_offset
            } else {
                vip_pos
            }
        }
    };
    let hold_radius = if cap.booster_radius > 0.0 {
        cap.booster_radius * cfg.support_orbit_factor
    } else {
        cfg.combat_tuning.support_default_radius
    };
    steer_arrive(pos, vel, anchor, hold_radius, decel, damping)
}

fn steer_arrive(
    pos: Vec2,
    vel: Vec2,
    target: Vec2,
    arrival_radius: f32,
    decel: f32,
    damping: f32,
) -> Steering {
    let delta = target - pos;
    let dist = delta.length();
    if dist <= arrival_radius {
        return Steering {
            desired_heading: None,
            thrust_world: Vec2::ZERO,
            throttle_cap: 0.0,
        };
    }
    let dir = delta / dist;
    let safe = (2.0 * decel.max(0.1) * (dist - arrival_radius)).sqrt();
    let desired = safe.min(decel / damping.max(0.1));
    let thrust = ((desired - vel.dot(dir)) / decel.max(0.1)).clamp(-1.0, 1.0);
    Steering {
        desired_heading: None,
        thrust_world: dir * thrust,
        throttle_cap: 1.0,
    }
}

fn resolve_contact_kinematics(
    target: Entity,
    contact: Option<SharedContact>,
    ship_transforms: &Query<(&GlobalTransform, &Ship)>,
) -> Option<(Vec2, Vec2, bool, Vec2)> {
    if let Some(c) = contact {
        return Some((c.position, c.velocity, c.has_fixed_gun, c.heading));
    }
    let (gt, ship) = ship_transforms.get(target).ok()?;
    let pos = gt.transform_point(ship.core_position.extend(0.0)).truncate();
    Some((pos, Vec2::ZERO, false, gt.up().truncate().normalize_or_zero()))
}

fn best_armor_heading(cap: &Capability, dir_to_enemy: Vec2, hp_ratio_threshold: f32) -> Option<f32> {
    let (best_q, &best_hp) = cap
        .quadrant_hp
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))?;
    let min_hp = cap
        .quadrant_hp
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    if best_hp <= min_hp * hp_ratio_threshold || best_hp <= 0.0 {
        return None;
    }
    let enemy_angle = dir_to_enemy.y.atan2(dir_to_enemy.x);
    // Quadrant 0 = +Y (front), 1 = +X (right, -FRAC_PI_2), 2 = -Y (back, PI), 3 = -X (left, +FRAC_PI_2)
    let quadrant_local_offset = match best_q {
        0 => 0.0,
        1 => -FRAC_PI_2,
        2 => PI,
        _ => FRAC_PI_2,
    };
    Some(enemy_angle - FRAC_PI_2 - quadrant_local_offset)
}

fn clamp_unit(v: Vec2) -> Vec2 {
    let len = v.length();
    if len > 1.0 { v / len } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn arrive_inside_radius_returns_zero_thrust() {
        let steer = steer_arrive(
            Vec2::ZERO,
            Vec2::X * 500.0,
            Vec2::X * 100.0,
            200.0,
            100.0,
            0.8,
        );
        assert_eq!(steer.thrust_world, Vec2::ZERO);
        assert_eq!(steer.throttle_cap, 0.0);
        assert!(steer.thrust_world.is_finite());
    }

    #[test]
    fn arrive_scales_desired_speed_with_decel() {
        let vel = Vec2::X * 125.0;
        let target = Vec2::X * 10_000.0;
        let fast = steer_arrive(Vec2::ZERO, vel, target, 100.0, 1000.0, 0.8);
        let slow = steer_arrive(Vec2::ZERO, vel, target, 100.0, 100.0, 0.8);
        assert!(fast.thrust_world.x > slow.thrust_world.x);
        assert!(slow.thrust_world.x.abs() < 1e-3);
    }

    #[test]
    fn arrive_brakes_with_reverse_thrust_when_overspeed() {
        let steer = steer_arrive(
            Vec2::ZERO,
            Vec2::X * 300.0,
            Vec2::X * 10_000.0,
            100.0,
            100.0,
            0.8,
        );
        assert!(steer.thrust_world.x < -0.9);
        assert_eq!(steer.throttle_cap, 1.0);
    }

    #[test]
    fn arrive_thrust_is_continuous_across_braking_boundary() {
        let mut previous = f32::NEG_INFINITY;
        for step in 0..20 {
            let dist = 160.0 + step as f32 * 2.0;
            let steer = steer_arrive(
                Vec2::ZERO,
                Vec2::X * 120.0,
                Vec2::X * dist,
                100.0,
                100.0,
                0.8,
            );
            let thrust = steer.thrust_world.x;
            assert!(thrust.is_finite());
            assert!(thrust >= previous - 1e-4);
            if previous.is_finite() {
                assert!((thrust - previous).abs() < 0.05);
            }
            previous = thrust;
        }
    }

    #[test]
    fn search_arrival_steering_uses_ship_stats() {
        let mut app = App::new();
        app.init_resource::<AiConfigValue>()
            .init_resource::<TacticalBoard>()
            .init_resource::<ControlledShip>()
            .insert_resource(GameConfigValue(GameConfig::default()))
            .init_resource::<Time>()
            .add_systems(Update, steer_ships);

        let waypoint = Vec2::X * 10_000.0;
        let velocity = Vec2::X * 125.0;
        let spawn_ship = |world: &mut World, mass: f32, thrust: f32| {
            world
                .spawn((
                    Brain::new(waypoint, 10.0),
                    Skill::new(8),
                    Capability::default(),
                    Readiness::default(),
                    Team::new(1),
                    Ship::default(),
                    ShipStats {
                        mass,
                        thrust_power: thrust,
                        ..default()
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                    Velocity::linear(velocity),
                    Steering::default(),
                    FireOrder::default(),
                ))
                .id()
        };

        let light = spawn_ship(app.world_mut(), 10.0, 10_000.0);
        let heavy = spawn_ship(app.world_mut(), 100.0, 10_000.0);

        app.update();

        let light_thrust = app.world().get::<Steering>(light).unwrap().thrust_world.x;
        let heavy_thrust = app.world().get::<Steering>(heavy).unwrap().thrust_world.x;
        assert!(light_thrust > heavy_thrust);
        assert_eq!(app.world().get::<Steering>(light).unwrap().throttle_cap, 1.0);
    }
}
