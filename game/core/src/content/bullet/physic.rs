use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    common::{damage::component::Damage, death::component::Death, lifetime::component::Lifetime},
    content::bullet::component::{Bullet, MissileHitbox},
};

pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet), Without<Death>>,
    missile_hitboxes: Query<(&MissileHitbox, &GlobalTransform)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e1) {
                handle_bullet_collision(&mut commands, bullet, &mut bulletc);
            } else if let Ok((bullet, mut bulletc)) = bullets.get_mut(*e2) {
                handle_bullet_collision(&mut commands, bullet, &mut bulletc);
            } else if let Ok((hitbox, transform)) = missile_hitboxes.get(*e1) {
                handle_missile_collision(&mut commands, hitbox, transform);
            } else if let Ok((hitbox, transform)) = missile_hitboxes.get(*e2) {
                handle_missile_collision(&mut commands, hitbox, transform);
            }
        }
    }
}

fn handle_bullet_collision(commands: &mut Commands, bullet: Entity, bulletc: &mut Bullet) {
    if bulletc.piercing <= 0 {
        commands
            .entity(bullet)
            .insert((Visibility::Hidden, Death))
            .remove::<Collider>()
            .remove::<CollisionGroups>()
            .remove::<ActiveEvents>();
    } else {
        bulletc.piercing -= 1;
    }
}

fn handle_missile_collision(
    commands: &mut Commands,
    hitbox: &MissileHitbox,
    transform: &GlobalTransform,
) {
    commands
        .entity(hitbox.missile)
        .insert((Visibility::Hidden, Death))
        .remove::<Collider>()
        .remove::<CollisionGroups>()
        .remove::<ActiveEvents>();

    commands.spawn((
        hitbox.team,
        Transform::from(*transform),
        Lifetime::seconds(0.1),
        Collider::ball(hitbox.blast_radius),
        CollisionGroups::new(hitbox.team.bullet_group(), hitbox.team.bullet_filter()),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        Damage {
            damage: hitbox.blast_damage,
        },
    ));
}
