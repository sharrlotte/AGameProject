use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    DamageMessage, EnemyTag, GROUP_ENEMY, GROUP_PROJECTILE, GROUP_WORLD, ObstacleTag, RunEntity,
};

use crate::config::WeaponSpecData;

use super::state::Projectile;

/// Spawns a single physical projectile sensor entity at `muzzle_pos`.
pub fn spawn_projectile(
    commands: &mut Commands,
    owner: Entity,
    muzzle_pos: Vec3,
    dir: Vec3,
    damage: f32,
    spec: &WeaponSpecData,
) -> Entity {
    commands
        .spawn((
            Name::new("Projectile"),
            RunEntity,
            Projectile {
                owner,
                damage,
                knockback: spec.knockback,
                direction: dir,
                radius: spec.projectile_radius,
                lifetime: spec.projectile_lifetime,
                hit: false,
            },
            RigidBody::Dynamic,
            GravityScale(0.0),
            Ccd::enabled(),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            Collider::ball(spec.projectile_radius),
            Velocity::linear(dir * spec.projectile_speed),
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
    enemies: Query<(), With<EnemyTag>>,
    obstacles: Query<(), With<ObstacleTag>>,
) {
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

        let Ok(mut proj) = projectiles.get_mut(proj_entity) else {
            continue;
        };
        if proj.hit {
            continue;
        }

        if enemies.contains(other_entity) {
            proj.hit = true;
            damage_writer.write(DamageMessage {
                target: other_entity,
                amount: proj.damage,
                knockback: proj.direction * proj.knockback,
                source: proj.owner,
            });
            commands.entity(proj_entity).despawn();
        } else if obstacles.contains(other_entity) {
            proj.hit = true;
            commands.entity(proj_entity).despawn();
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
        if proj.lifetime <= 0.0 && !proj.hit {
            commands.entity(entity).despawn();
        }
    }
}
