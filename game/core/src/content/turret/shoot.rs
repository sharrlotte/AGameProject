use std::f32::consts::PI;

use crate::components::damage::Damage;
use crate::components::lifetime::Lifetime;
use crate::content::block::base::{
    ShootingProjectile, SingleShootPattern, SingleShootPatternState,
};
use crate::content::bullet::base::{Bullet, Missile};
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
    position: Vec2,
    range: f32,
    projectile: &ShootingProjectile,
) {
    let rotation: Quat = turret_transform.rotation();
    let dir = rotation.to_euler(EulerRot::XYZ).2 + PI / 2.0;

    let (_, turret_rot, _) = turret_transform.to_scale_rotation_translation();
    let world_offset = turret_rot * position.extend(0.0);
    let spawn_position = turret_transform.translation() + world_offset;

    match &projectile {
        ShootingProjectile::Bullet {
            speed,
            damage,
            lifetime,
            piercing,
        } => {
            let bullet = commands
                .spawn((
                    Lifetime {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
                    },
                    Bullet {
                        shooted_at: spawn_position.truncate(),
                        range,
                        piercing: *piercing,
                        velocity: Vec2::from_angle(dir) * *speed,
                    },
                    Damage { damage: *damage },
                    Sprite {
                        color: if team.is_player() {
                            Color::srgb(10.0, 0.0, 0.0)
                        } else {
                            Color::srgb(0.0, 10.0, 0.0)
                        },
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
        } => {
            let missile = commands
                .spawn((
                    Lifetime {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
                    },
                    Missile {
                        shooted_at: spawn_position.truncate(),
                        range,
                        speed: *speed,
                        target: None,
                    },
                    Sprite {
                        color: if team.is_player() {
                            Color::srgb(8.0, 0.0, 10.0)
                        } else {
                            Color::srgb(8.0, 10.0, 0.0)
                        },
                        custom_size: Some(Vec2::new(2.0, 50.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    Collider::ball(range),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    *team,
                ))
                .id();

            let hit_box = commands
                .spawn((
                    Collider::ball(5.0),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    Damage { damage: *damage },
                ))
                .id();

            commands.entity(missile).add_child(hit_box);
        }
    }
}
