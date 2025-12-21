use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent, ExternalForce, RigidBody};

use crate::{
    components::{
        damage::{Damage, DamageEvent},
        death::Death,
    },
    content::{
        block::base::Health,
        bullet::base::{Bullet, Missile},
    },
};

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
    mut writer: MessageWriter<DamageEvent>,
    damages: Query<&Damage>,
    healths: Query<&Health>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Some((attacker, target)) = match (
                damages.contains(*e1),
                healths.contains(*e2),
                damages.contains(*e2),
                healths.contains(*e1),
            ) {
                (true, true, _, _) => Some((e1, e2)),
                (_, _, true, true) => Some((e2, e1)),
                _ => None,
            } {
                if let Ok(damage) = damages.get(*attacker) {
                    writer.write(DamageEvent {
                        attacker: *attacker,
                        target: *target,
                        damage: damage.damage,
                    });
                } else {
                    warn!("Missing damage {} or {}, idk how", e1, e2);
                }
            }

            let b1 = bullets.get(*e1);
            let b2 = bullets.get(*e2);

            let Ok((bullet, _)) = b1.or(b2) else {
                continue;
            };

            let Ok((_, mut bulletc)) = bullets.get_mut(bullet) else {
                warn!("Missing damage {} or {}, idk how", e1, e2);
                continue;
            };

            if bulletc.piercing <= 0 {
                commands
                    .entity(bullet)
                    .insert(Visibility::Hidden)
                    .remove::<Collider>()
                    .remove::<ActiveEvents>()
                    .remove::<RigidBody>();
            } else {
                bulletc.piercing -= 1;
            }
        }
    }
}

pub fn handle_bullets(mut query: Query<(&mut Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut transform) in query.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_secs();
    }
}

pub fn handle_despawn_bullet(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &GlobalTransform)>,
) {
    for (entity, bullet, global_transform) in query.iter_mut() {
        if bullet.range < (bullet.shooted_at - global_transform.translation().truncate()).length() {
            commands.entity(entity).insert(Death);
            continue;
        }
    }
}

pub fn handle_missile_radar(
    mut collision_events: MessageReader<CollisionEvent>,
    mut missiles: Query<&mut Missile>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok(mut missile) = missiles.get_mut(*e1) {
                missile.target = Some(*e2);
            } else if let Ok(mut missile) = missiles.get_mut(*e2) {
                missile.target = Some(*e1);
            }
        }
    }
}

pub fn handle_missiles(
    mut missiles: Query<(&Missile, &mut Transform, &GlobalTransform)>,
    targets: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (missile, mut transform, global) in missiles.iter_mut() {
        let forward = global.up().truncate();

        if let Some(target) = missile.target {
            let Ok(target_gt) = targets.get(target) else {
                warn!("Missing target possition");
                continue;
            };

            let to_target = target_gt.translation().truncate() - global.translation().truncate();

            if to_target.length_squared() < 0.0001 {
                continue;
            }

            let target_dir = to_target.normalize();
            let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
            let max_turn = 2.0 * time.delta_secs();
            let turn = angle_error.clamp(-max_turn, max_turn);

            transform.rotate_z(turn);
            transform.translation += (forward * missile.speed * time.delta_secs()).extend(0.0);
        };

        transform.translation += (forward * missile.speed * time.delta_secs()).extend(0.0);
    }
}
