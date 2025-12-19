use bevy::prelude::*;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent, RigidBody};

use crate::{
    components::death::Death,
    content::{
        block::{
            base::{BelongsToShip, Block, BlockGroup, Health},
            registry::BlockRegistry,
        },
        bullet::base::Bullet,
        ship::base::{RebuildShipCollider, Ship},
    },
};

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
    mut healths: Query<(Entity, &mut Health, &mut Block, &BelongsToShip)>,
    mut ships: Query<&mut Ship>,
    registry: Res<BlockRegistry>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let b1 = bullets.get(*e1);
            let b2 = bullets.get(*e2);

            let Ok((bullet, _)) = b1.or(b2) else {
                continue;
            };

            let Ok((_, mut bulletc)) = bullets.get_mut(bullet) else {
                warn!("Missing bullet {} or {}, idk how", e1, e2);
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

            let h1 = healths.contains(*e1);
            let h2 = healths.contains(*e2);

            let health = if h1 {
                *e1
            } else if h2 {
                *e2
            } else {
                continue;
            };

            let Ok((_, mut healthc, block, belong)) = healths.get_mut(health) else {
                warn!("Missing health {} or {}, idk how", e1, e2);
                continue;
            };

            healthc.health -= bulletc.damage;

            if healthc.health <= 0.0 {
                commands.entity(health).insert(Death);

                if ships.contains(belong.ship) {
                    commands.entity(belong.ship).insert(RebuildShipCollider);
                }

                if let Ok(mut ship) = ships.get_mut(belong.ship) {
                    ship.blocks.retain_mut(|f| f.id != block.id);
                }

                let Some(def) = registry.get(&block.block_type) else {
                    continue;
                };

                if def.block_group != BlockGroup::Core {
                    continue;
                }

                if ships.contains(belong.ship) {
                    commands.entity(belong.ship).insert(Death);
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
            commands.entity(entity).insert(Death);
            continue;
        }

        if bullet.range < (bullet.shooted_at - global_transform.translation().truncate()).length() {
            commands.entity(entity).insert(Death);
            continue;
        }
    }
}
