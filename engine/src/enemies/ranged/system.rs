use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::common::{DamageKind, Dying, GROUP_PROJECTILE, GROUP_WORLD, GameScene, RunEntity};
use crate::config::EnemiesConfig;
use crate::hazard::{HazardSpec, HazardSpawnMessage};
use crate::world::{PlayerConfig, Survivor};

use crate::enemies::ai::state::Enemy;
use crate::enemies::brain::{BrainState, EnemyBrain};

use super::state::{ACID_MAX_FLIGHT_SECS, ACID_PROJECTILE_RADIUS, AcidProjectile, Ranged};

const GROUND_Y: f32 = 0.0;

/// Resolves rooted wind-ups, reach checks, cooldown gating, and acid launches.
#[allow(clippy::type_complexity)]
pub fn enemy_ranged(
    time: Res<Time>,
    player_cfg: Res<PlayerConfig>,
    enemies_cfg: Res<EnemiesConfig>,
    mut commands: Commands,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut attackers: Query<
        (
            Entity,
            &Enemy,
            &Transform,
            &EnemyBrain,
            &mut Ranged,
            Has<Dying>,
        ),
        Without<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let target = survivor.as_ref().map(|single| {
        let (entity, transform) = **single;
        (entity, transform.translation)
    });

    for (_entity, enemy, transform, brain, mut ranged, dying) in &mut attackers {
        if dying {
            continue;
        }
        ranged.cooldown_timer = (ranged.cooldown_timer - dt).max(0.0);

        if ranged.is_winding() {
            ranged.windup_timer = (ranged.windup_timer - dt).max(0.0);
            if ranged.windup_timer <= 0.0 {
                ranged.cooldown_timer = ranged.cooldown;
                if let Some((target_entity, target_pos)) = target
                    && in_reach(player_cfg.radius, enemy.radius, &ranged, transform.translation, target_pos)
                    && brain.target == Some(target_entity)
                {
                    fire(
                        &mut commands,
                        transform.translation,
                        target_pos,
                        &ranged,
                        enemies_cfg.acid_gravity,
                    );
                }
            }
            continue;
        }

        if brain.state != BrainState::Engage || ranged.cooldown_timer > 0.0 {
            continue;
        }
        let Some((target_entity, target_pos)) = target else {
            continue;
        };
        if brain.target != Some(target_entity)
            || !in_reach(player_cfg.radius, enemy.radius, &ranged, transform.translation, target_pos)
        {
            continue;
        }

        if ranged.windup > 0.0 {
            ranged.windup_timer = ranged.windup;
        } else {
            ranged.cooldown_timer = ranged.cooldown;
            fire(
                &mut commands,
                transform.translation,
                target_pos,
                &ranged,
                enemies_cfg.acid_gravity,
            );
        }
    }
}

fn fire(
    commands: &mut Commands,
    from: Vec3,
    target: Vec3,
    ranged: &Ranged,
    gravity: f32,
) {
    let ground_target = Vec3::new(target.x, GROUND_Y, target.z);
    let Some(velocity) = launch_velocity(
        from,
        ground_target,
        ranged.projectile_speed,
        ranged.arc_deg,
        gravity,
    ) else {
        return;
    };
    spawn_acid_projectile(commands, from, velocity, ranged);
}

/// Solves the launch velocity that lands `from` at `target` for an authored elevation arc.
pub fn launch_velocity(
    from: Vec3,
    target: Vec3,
    speed_cap: f32,
    arc_deg: f32,
    gravity: f32,
) -> Option<Vec3> {
    let flat = Vec3::new(target.x - from.x, 0.0, target.z - from.z);
    let distance = flat.length();
    if distance <= 1e-3
        || !speed_cap.is_finite()
        || speed_cap <= 0.0
        || !gravity.is_finite()
        || gravity <= 0.0
    {
        return None;
    }
    let direction = flat / distance;
    let rise = target.y - from.y;
    let angle = arc_deg.clamp(1.0, 89.0).to_radians();
    let cos = angle.cos();
    let denominator = 2.0 * cos * cos * (distance * angle.tan() - rise);
    let solved = if denominator > 1e-4 {
        (gravity * distance * distance / denominator).sqrt()
    } else {
        speed_cap
    };
    let speed = if solved.is_finite() && solved > 0.0 {
        solved.min(speed_cap)
    } else {
        speed_cap
    };
    if !speed.is_finite() || speed <= 0.0 {
        return None;
    }
    Some(Vec3::new(
        direction.x * speed * cos,
        speed * angle.sin(),
        direction.z * speed * cos,
    ))
}

pub(super) fn spawn_acid_projectile(
    commands: &mut Commands,
    from: Vec3,
    velocity: Vec3,
    ranged: &Ranged,
) -> Entity {
    commands
        .spawn((
            Name::new("AcidProjectile"),
            RunEntity,
            DespawnOnExit(GameScene::Arena),
            AcidProjectile {
                damage: ranged.damage,
                pool_radius: ranged.pool_radius,
                pool_lifetime: ranged.pool_lifetime,
                lifetime: ACID_MAX_FLIGHT_SECS,
            },
            RigidBody::Dynamic,
            GravityScale(0.0),
            Ccd::enabled(),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            Collider::ball(ACID_PROJECTILE_RADIUS),
            Velocity::linear(velocity),
            TransformInterpolation::default(),
            Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            CollisionGroups::new(GROUP_PROJECTILE, GROUP_WORLD),
            Transform::from_translation(from),
        ))
        .id()
}

/// Applies authored gravity because the physics world runs with gravity disabled.
pub fn fall_acid_projectiles(
    time: Res<Time>,
    cfg: Res<EnemiesConfig>,
    mut projectiles: Query<&mut Velocity, With<AcidProjectile>>,
) {
    let dt = time.delta_secs();
    for mut velocity in &mut projectiles {
        velocity.linear.y -= cfg.acid_gravity * dt;
    }
}

/// Spawns one pool when a projectile hits world geometry, reaches the ground, or expires.
pub fn resolve_acid_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut collision_reader: MessageReader<CollisionEvent>,
    mut hazard_writer: MessageWriter<HazardSpawnMessage>,
    mut projectiles: Query<(Entity, &Transform, &mut AcidProjectile)>,
) {
    let dt = time.delta_secs();
    let mut collided: HashSet<Entity> = HashSet::new();
    for event in collision_reader.read() {
        let CollisionEvent::Started(first, second, _) = *event else {
            continue;
        };
        if projectiles.contains(first) {
            collided.insert(first);
        } else if projectiles.contains(second) {
            collided.insert(second);
        }
    }

    for (entity, transform, mut projectile) in &mut projectiles {
        projectile.lifetime -= dt;
        let grounded = transform.translation.y <= GROUND_Y;
        if !collided.contains(&entity) && !grounded && projectile.lifetime > 0.0 {
            continue;
        }
        hazard_writer.write(HazardSpawnMessage {
            position: transform.translation,
            spec: HazardSpec {
                radius: projectile.pool_radius,
                damage: projectile.damage,
                lifetime: projectile.pool_lifetime,
                kind: DamageKind::Acid,
            },
        });
        commands.entity(entity).despawn();
    }
}

fn in_reach(player_radius: f32, enemy_radius: f32, ranged: &Ranged, from: Vec3, target: Vec3) -> bool {
    flat_distance(from, target) <= player_radius + enemy_radius + ranged.reach
}

fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}
