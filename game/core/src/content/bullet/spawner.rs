use std::f32::consts::PI;

use crate::content::bullet::types::Bullet;
use crate::content::team::Team;
use crate::content::turret::types::ShootingProjectile;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn spawn_bullet(
    commands: &mut Commands,
    turret_transform: &GlobalTransform,
    team: &Team,
    barrel_pos: Vec2,
    range: f32,
    projectile: &ShootingProjectile,
) {
    let rotation: Quat = turret_transform.rotation();
    let dir = rotation.to_euler(EulerRot::XYZ).2 + PI / 2.0;

    // Transform offset by turret rotation
    // We need the turret's current rotation from GlobalTransform
    let (_, turret_rot, _) = turret_transform.to_scale_rotation_translation();
    let world_offset = turret_rot * barrel_pos.extend(0.0);
    let spawn_pos = turret_transform.translation() + world_offset;

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
                        shooted_at: spawn_pos.truncate(),
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
                        translation: spawn_pos,
                        rotation,
                        ..default()
                    },
                    RigidBody::Dynamic,
                    Collider::ball(5.0),
                    CollisionGroups::new(team.get_bullet_group(), team.get_bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Velocity::linear(Vec2::from_angle(dir) * *speed),
                    GravityScale(0.0),
                    ColliderMassProperties::Mass(0.01),
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
