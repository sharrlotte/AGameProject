use std::f32::consts::PI;

use crate::content::block::base::{ShootPattern, ShootPatternState, ShootingProjectile};
use crate::content::bullet::base::Bullet;
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
        &ShootPattern,
        &mut ShootPatternState,
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

        match (shoot_pattern, &mut *shoot_pattern_state) {
            (
                ShootPattern::Single {
                    fire_rate,
                    projectile,
                },
                ShootPatternState::Single { last_fired_at },
            ) => {
                if time.elapsed_secs() - *last_fired_at >= *fire_rate {
                    for barrel_position in barrels.iter() {
                        spawn_bullet(
                            &mut commands,
                            transform,
                            team,
                            *barrel_position,
                            *range,
                            &projectile,
                        );
                    }
                    *last_fired_at = time.elapsed_secs();
                }
            }
            _ => {
                warn!(
                    "Unknown shoot pattern {:?} {:?}",
                    shoot_pattern, shoot_pattern_state
                );
            }
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
                    Bullet {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
                        shooted_at: spawn_position.truncate(),
                        range,
                        damage: *damage,
                        piercing: *piercing,
                    },
                    Sprite {
                        color: Color::srgb(10.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(2.0, 50.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    RigidBody::Dynamic,
                    Collider::ball(5.0),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Velocity::linear(Vec2::from_angle(dir) * *speed),
                    GravityScale(0.0),
                    ColliderMassProperties::Mass(0.001),
                    Sensor,
                    Damping {
                        linear_damping: 0.1,
                        angular_damping: 0.1,
                    },
                    *team,
                ))
                .id();

            if (*piercing) > 0 {
                commands.entity(bullet).insert(Sensor);
            }
        }
        ShootingProjectile::Beam { .. } => {
            // Beam implementation omitted for brevity
        }
    }
}
