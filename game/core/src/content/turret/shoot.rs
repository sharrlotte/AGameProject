use crate::components::damage::Damage;
use crate::components::entity_type::EntityType;
use crate::components::lifetime::Lifetime;
use crate::content::block::base::{
    ShootingProjectile, SingleShootPattern, SingleShootPatternState,
};
use crate::content::bullet::base::{Bullet, Missile, MissileHitbox};
use crate::content::team::Team;
use crate::content::turret::base::{Turret, TurretState};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn handle_turret_emission(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        &GlobalTransform,
        &Turret,
        &mut TurretState,
        &SingleShootPattern,
        &mut SingleShootPatternState,
        &Team,
    )>,
) {
    for (transform, turret, turret_state, shoot_pattern, mut shoot_pattern_state, team) in
        query.iter_mut()
    {
        if !turret_state.active {
            continue;
        }

        let Turret { barrels, range, .. } = turret;

        if time.elapsed_secs() - shoot_pattern_state.last_fired_at >= shoot_pattern.fire_rate {
            for barrel_position in barrels.iter() {
                spawn_bullet(
                    &mut commands,
                    transform,
                    team,
                    *barrel_position,
                    *range,
                    &shoot_pattern.projectile,
                    turret_state.target,
                );
            }
            shoot_pattern_state.last_fired_at = time.elapsed_secs();
        }
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    turret_transform: &GlobalTransform,
    team: &Team,
    barrel_position: Vec2,
    range: f32,
    projectile: &ShootingProjectile,
    target: Option<Entity>,
) {
    let rotation = turret_transform.rotation();
    let world_offset = rotation * barrel_position.extend(0.0);
    let spawn_position = turret_transform.translation() + world_offset;
    let dir = (rotation * Vec3::Y).truncate().normalize();

    match &projectile {
        ShootingProjectile::Bullet {
            speed,
            damage,
            lifetime,
            piercing,
        } => {
            let bullet = commands
                .spawn((
                    EntityType::Projectile,
                    Lifetime {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
                    },
                    Bullet {
                        shooted_at: spawn_position.truncate(),
                        range,
                        piercing: *piercing,
                        velocity: dir * *speed,
                    },
                    Damage { damage: *damage },
                    Sprite {
                        color: team.color(10.0),
                        custom_size: Some(Vec2::new(2.0, 50.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    Collider::ball(5.0),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    *team,
                ))
                .id();

            if (*piercing) > 0 {
                commands.entity(bullet).insert(Sensor);
            }
        }
        ShootingProjectile::Beam { .. } => {
            todo!("Beam implementation");
        }
        ShootingProjectile::Missile {
            speed,
            damage,
            lifetime,
            lock_range,
        } => {
            let missile = commands
                .spawn((
                    EntityType::Projectile,
                    Lifetime {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
                    },
                    Missile {
                        shooted_at: spawn_position.truncate(),
                        lock_range: *lock_range,
                        speed: *speed,
                        target,
                    },
                    Sprite {
                        color: team.color(10.0),
                        custom_size: Some(Vec2::new(5.0, 50.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    *team,
                ))
                .id();

            if target.is_some() {
                commands.entity(missile).insert((
                    Collider::ball(*lock_range),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                ));
            }

            let hit_box = commands
                .spawn((
                    Collider::ball(5.0),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    Damage { damage: *damage },
                    MissileHitbox { missile },
                    Transform::default(),
                ))
                .id();

            commands.entity(missile).add_child(hit_box);
        }
    }
}
