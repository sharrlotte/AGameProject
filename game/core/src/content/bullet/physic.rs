use bevy::prelude::*;
use bevy_rapier2d::prelude::{ColliderDisabled, CollisionEvent};

use crate::{
    components::death::Death,
    content::bullet::base::{Bullet, Missile, MissileHitbox},
};

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
    mut missile_radars: Query<&mut Missile>,
    missile_hitboxes: Query<(Entity, &MissileHitbox)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e1) {
                handle_bullet_collision(&mut commands, bullet, &mut bulletc);
            } else if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e2) {
                handle_bullet_collision(&mut commands, bullet, &mut bulletc);
            }

            if let Ok((missile, _)) = missile_hitboxes.get(*e1) {
                handle_missile_collision(&mut commands, missile);
            } else if let Ok((missile, _)) = missile_hitboxes.get(*e2) {
                handle_missile_collision(&mut commands, missile);
            }

            if let Ok(mut missilec) = missile_radars.get_mut(*e1) {
                handle_missile_radar_collision(&mut missilec, *e2);
            } else if let Ok(mut missilec) = missile_radars.get_mut(*e2) {
                handle_missile_radar_collision(&mut missilec, *e1);
            }
        }
    }
}

fn handle_bullet_collision(commands: &mut Commands, bullet: Entity, bulletc: &mut Bullet) {
    if bulletc.piercing <= 0 {
        commands
            .entity(bullet)
            .insert((Visibility::Hidden, ColliderDisabled, Death));
    } else {
        bulletc.piercing -= 1;
    }
}

fn handle_missile_collision(commands: &mut Commands, missile: Entity) {
    commands
        .entity(missile)
        .insert((Visibility::Hidden, ColliderDisabled, Death));
    info!("Missile hit");
}

fn handle_missile_radar_collision(missile: &mut Missile, target: Entity) {
    missile.target = Some(target);
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
            if let Ok(target_gt) = targets.get(target) {
                let to_target =
                    target_gt.translation().truncate() - global.translation().truncate();

                if to_target.length_squared() < 0.0001 {
                    continue;
                }

                let target_dir = to_target.normalize();
                let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
                let max_turn = 2.0 * time.delta_secs();
                let turn = angle_error.clamp(-max_turn, max_turn);

                transform.rotate_z(turn);
                transform.translation += (forward * missile.speed * time.delta_secs()).extend(0.0);

                continue;
            };
        };

        transform.translation += (forward * missile.speed * time.delta_secs()).extend(0.0);
    }
}
