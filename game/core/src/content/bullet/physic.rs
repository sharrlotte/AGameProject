use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, ExternalForce, RigidBody, Velocity,
};

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
            }
        }
    }
}

pub fn handle_missiles(
    mut query: Query<(
        &mut Missile,
        &GlobalTransform,
        &mut ExternalForce,
        &mut Velocity,
    )>,
    targets: Query<&GlobalTransform>,
) {
    for (missile, transform, mut force, mut velocity) in query.iter_mut() {
        let Some(target) = missile.target else {
            continue;
        };

        let Ok(target_position) = targets.get(target) else {
            continue;
        };

        let direction = (target_position.translation().truncate()
            - transform.translation().truncate())
        .normalize_or_zero();

        let angle = direction.y.atan2(direction.x) - PI / 2.0;
        let angle_diff = angle - transform.rotation().to_euler(EulerRot::XYZ).2;
        let turn_speed_multiplier = (angle_diff.abs() / PI).clamp(0.25, 1.0);

        let torque = (if angle_diff > 0.0 { 1.0 } else { -1.0 }) * turn_speed_multiplier;

        velocity.angvel = velocity.angvel.clamp(-1.0, 1.0);

        force.torque = torque;
        force.force = direction * missile.speed;
    }
}
