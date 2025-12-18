use bevy::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent};

use crate::content::{bullet::Bullet, structures::components::Health};

pub fn handle_despawn_bullet(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &GlobalTransform)>,
) {
    for (entity, mut bullet, global_transform) in query.iter_mut() {
        bullet.timer.tick(time.delta());

        if bullet.timer.is_finished() {
            commands.entity(entity).despawn();
            return;
        }

        if bullet.range < (bullet.shooted_at - global_transform.translation().truncate()).length() {
            commands.entity(entity).despawn();
            return;
        }
    }
}

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
    mut healths: Query<(Entity, &mut Health)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e1) {
                if let Ok((_, mut healthc)) = healths.get_mut(*e2) {
                    healthc.health -= bulletc.damage;

                    handle_bullet(&mut commands, bullet, &mut bulletc);
                }
                continue;
            }

            if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e2) {
                if let Ok((_, mut healthc)) = healths.get_mut(*e1) {
                    healthc.health -= bulletc.damage;
                }

                handle_bullet(&mut commands, bullet, &mut bulletc);
            }
        }
    }
}

fn handle_bullet(commands: &mut Commands, bullet: Entity, bulletc: &mut Bullet) {
    if bulletc.piercing <= 0 {
        commands
            .entity(bullet)
            .insert(Visibility::Hidden)
            .remove::<Collider>()
            .remove::<ActiveEvents>();
    } else {
        bulletc.piercing -= 1;
    }
}
