use bevy::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent};

use crate::content::{
    bullet::Bullet,
    structures::{
        block::{Block, BlockGroup},
        components::Health,
        ship::Ship,
    },
};

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
    mut healths: Query<(Entity, &mut Health, &mut Block)>,
    parents: Query<&ChildOf>,
    ships: Query<&Ship>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (bullet_entity, health_entity) = match (
                bullets.contains(*e1),
                healths.contains(*e1),
                bullets.contains(*e2),
                healths.contains(*e2),
            ) {
                (true, _, _, true) => (bullets.get_mut(*e1), healths.get_mut(*e2)),
                (_, true, true, _) => (bullets.get_mut(*e2), healths.get_mut(*e1)),
                _ => continue,
            };

            if let (Ok((bullet, mut bulletc)), Ok((health, mut healthc, block))) =
                (bullet_entity, health_entity)
            {
                healthc.health -= bulletc.damage;

                if healthc.health <= 0.0 {
                    commands.entity(health).despawn();

                    if block.def.block_group == BlockGroup::Core {
                        if let Ok(parent) = parents.get(health) {
                            if let Ok(_ship) = ships.get(parent.0) {
                                commands.entity(parent.0).despawn();
                            }
                        }
                    }
                }

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
        }
    }
}

pub fn handle_despawn_bullet(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &GlobalTransform)>,
) {
    for (entity, mut bullet, global_transform) in query.iter_mut() {
        bullet.timer.tick(time.delta());

        if bullet.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        if bullet.range < (bullet.shooted_at - global_transform.translation().truncate()).length() {
            commands.entity(entity).despawn();
            continue;
        }
    }
}
