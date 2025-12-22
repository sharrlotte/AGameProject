use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{damage::Damage, death::Death},
    content::bullet::base::{Bullet, Lifetime, Missile, MissileHitbox},
};
pub struct PhysicPlugin;

impl Plugin for PhysicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_collisions);
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet), Without<Death>>,
    mut missile_radars: Query<(Entity, &mut Missile), Without<Death>>,
    missile_hitboxes: Query<(&MissileHitbox, &GlobalTransform)>,
) {
    if collision_events.len() > 0 {
        info!("Collision event: {}", collision_events.len());
    }

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
            } else if let Ok((radar, mut missilec)) = missile_radars.get_mut(*e1) {
                handle_missile_radar_collision(&mut commands, radar, &mut missilec, *e2);
            } else if let Ok((radar, mut missilec)) = missile_radars.get_mut(*e2) {
                handle_missile_radar_collision(&mut commands, radar, &mut missilec, *e1);
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
        Transform::from(*transform),
        Lifetime::new(1.0),
        Damage {
            damage: hitbox.blast_damage,
        },
        Collider::ball(hitbox.blast_radius),
        CollisionGroups::new(
            hitbox.team.get_bullet_group(),
            hitbox.team.get_bullet_filter(),
        ),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
        hitbox.team,
    ));
}

fn handle_missile_radar_collision(
    commands: &mut Commands,
    radar: Entity,
    missile: &mut Missile,
    target: Entity,
) {
    if missile.target.is_none() {
        missile.target = Some(target);
    }

    commands.entity(radar).insert(ColliderDisabled);
}
