use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::common::{
    Dying, GROUP_ENEMY, GROUP_PROJECTILE, GROUP_WORLD, Obstacle, RunEntity,
};
use crate::enemies::Enemy;
use crate::message::messages::DamageMessage;
use crate::config::WeaponDefinition;

use super::state::{HitLog, Projectile};

/// Candidate collision contact for a projectile in the current frame.
struct Candidate {
    projectile: Entity,
    target: Entity,
    distance: f32,
    is_obstacle: bool,
}

/// Spawns a single physical projectile sensor entity at `muzzle_pos`.
pub fn spawn_projectile(
    commands: &mut Commands,
    owner: Entity,
    muzzle_pos: Vec3,
    dir: Vec3,
    damage: f32,
    weapon: &WeaponDefinition,
) -> Entity {
    commands
        .spawn((
            Name::new("Projectile"),
            RunEntity,
            Projectile {
                owner,
                damage,
                initial_damage: damage,
                remaining_power: weapon.pierce.power,
                penalty: weapon.pierce.penalty,
                knockback: weapon.knockback,
                direction: dir,
                radius: weapon.projectile_radius,
                lifetime: weapon.projectile_lifetime,
                kind: weapon.damage_kind,
                hit_log: HitLog::new(),
            },
            RigidBody::Dynamic,
            GravityScale(0.0),
            Ccd::enabled(),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            Collider::ball(weapon.projectile_radius),
            Velocity::linear(dir * weapon.projectile_speed),
            TransformInterpolation::default(),
            LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED,
            CollisionGroups::new(GROUP_PROJECTILE, GROUP_WORLD | GROUP_ENEMY),
            Transform::from_translation(muzzle_pos),
        ))
        .id()
}

/// Resolves sensor collision events for 3D projectiles against enemies and arena obstacles.
pub fn handle_projectile_collisions(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionEvent>,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut projectiles: Query<&mut Projectile>,
    enemies: Query<&Enemy, Without<Dying>>,
    obstacles: Query<(), With<Obstacle>>,
    transforms: Query<&Transform>,
) {
    let mut candidates: Vec<Candidate> = Vec::new();

    for event in collision_reader.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            continue;
        };

        let (proj_entity, other_entity) = if projectiles.contains(e1) {
            (e1, e2)
        } else if projectiles.contains(e2) {
            (e2, e1)
        } else {
            continue;
        };

        let is_obstacle = obstacles.contains(other_entity);
        if !is_obstacle && !enemies.contains(other_entity) {
            continue;
        }

        // Avoid adding duplicate pair candidates from Rapier in the same frame
        if candidates
            .iter()
            .any(|c| c.projectile == proj_entity && c.target == other_entity)
        {
            continue;
        }

        let proj_pos = transforms
            .get(proj_entity)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);
        let target_pos = transforms
            .get(other_entity)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

        let proj_dir = projectiles
            .get(proj_entity)
            .map(|p| p.direction)
            .unwrap_or(Vec3::ZERO);

        let distance = if proj_dir.is_finite() && proj_dir.length_squared() > 1e-6 {
            (target_pos - proj_pos).dot(proj_dir)
        } else {
            0.0
        };

        candidates.push(Candidate {
            projectile: proj_entity,
            target: other_entity,
            distance,
            is_obstacle,
        });
    }

    if candidates.is_empty() {
        return;
    }

    // Sort by projectile, then by distance along travel direction, then by entity index for determinism
    candidates.sort_by(|a, b| {
        a.projectile
            .cmp(&b.projectile)
            .then_with(|| {
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.target.cmp(&b.target))
    });

    let mut current_proj: Option<Entity> = None;
    let mut should_despawn = false;

    for candidate in candidates {
        if current_proj != Some(candidate.projectile) {
            if let Some(prev) = current_proj {
                if should_despawn {
                    commands.entity(prev).despawn();
                }
            }
            current_proj = Some(candidate.projectile);
            should_despawn = false;
        }

        if should_despawn {
            continue;
        }

        let Ok(mut proj) = projectiles.get_mut(candidate.projectile) else {
            continue;
        };

        if candidate.is_obstacle {
            should_despawn = true;
            continue;
        }

        let Ok(enemy) = enemies.get(candidate.target) else {
            continue;
        };

        if proj.hit_log.contains(candidate.target) {
            continue;
        }

        let log_is_full = proj.hit_log.push(candidate.target);

        let knockback_fraction = if proj.initial_damage > 0.0 {
            (proj.damage / proj.initial_damage).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let scaled_knockback = proj.direction * (proj.knockback * knockback_fraction);

        damage_writer.write(DamageMessage {
            target: candidate.target,
            amount: proj.damage,
            knockback: scaled_knockback,
            source: proj.owner,
            kind: proj.kind,
            hit_dir: proj.direction,
        });

        // Deal damage before subtracting cost and penalty
        proj.remaining_power -= enemy.pierce_cost;
        proj.damage = (proj.damage - enemy.pierce_cost * proj.penalty).max(0.0);

        let direction_invalid =
            !proj.direction.is_finite() || proj.direction.length_squared() <= 1e-6;

        if direction_invalid
            || proj.remaining_power <= 0.0
            || proj.damage <= 0.0
            || log_is_full
        {
            should_despawn = true;
        }
    }

    if let Some(prev) = current_proj {
        if should_despawn {
            commands.entity(prev).despawn();
        }
    }
}

/// Despawns projectiles whose maximum flight lifetime has elapsed.
pub fn tick_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut proj) in &mut projectiles {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
