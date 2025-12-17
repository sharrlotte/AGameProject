use crate::content::bullet::Bullet;
use crate::content::team::Team;
use crate::content::turret::{ShootPattern, ShootingProjectile, TurretTargeter};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*; // Reusing existing Bullet component if possible, or define new one.

pub fn handle_turret_emission(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &mut TurretTargeter, Option<&Team>)>,
) {
    for (transform, mut targeter, team) in query.iter_mut() {
        if !targeter.active {
            continue;
        }

        let Some(target_pos) = targeter.target_pos else {
            continue;
        };

        let TurretTargeter {
            shooting_pattern,
            barrels,
            range,
            ..
        } = &mut *targeter;

        match shooting_pattern {
            ShootPattern::Single {
                fire_rate,
                last_fired_at,
                projectile,
            } => {
                if time.elapsed_secs() - *last_fired_at >= *fire_rate {
                    for barrel_pos in barrels.iter_mut() {
                        fire_projectiles(
                            &mut commands,
                            transform,
                            team,
                            target_pos,
                            *barrel_pos,
                            *range,
                            &projectile,
                        );
                    }
                    *last_fired_at = time.elapsed_secs();
                }
            }
            ShootPattern::Burst {
                count,
                delay,
                current_shot,
                burst_timer,
                is_bursting,
                projectile,
            } => {}
            ShootPattern::Continuous => {}
        }
    }
}

fn fire_projectiles(
    commands: &mut Commands,
    turret_transform: &GlobalTransform,
    team: Option<&Team>,
    target_pos: Vec2,
    barrel_pos: Vec2,
    range: f32,
    projectile: &ShootingProjectile,
) {
    let turret_pos = turret_transform.translation().truncate();
    let dir_to_target = (target_pos - turret_pos).normalize_or_zero();
    let angle = Vec2::Y.angle_to(dir_to_target);
    let rotation = Quat::from_rotation_z(angle);

    let team_comp = team.copied().unwrap_or_default();

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
                        color: Color::srgb(1.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(5.0, 10.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_pos,
                        rotation,
                        ..default()
                    },
                    RigidBody::Dynamic,
                    Collider::ball(10.0),
                    Velocity::linear(dir_to_target * *speed),
                    ActiveEvents::COLLISION_EVENTS,
                    GravityScale(0.0),
                    CollisionGroups::new(Group::default(), team_comp.get_bullet_filter()),
                    ColliderMassProperties::Mass(0.01),
                    team_comp,
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

pub fn handle_despawn_bullet(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &GlobalTransform)>,
) {
    for (entity, mut bullet, global_transform) in query.iter_mut() {
        bullet.timer.tick(time.delta());

        if bullet.timer.is_finished() {
            commands.entity(entity).despawn();
        }

        if bullet.range < (bullet.shooted_at - global_transform.translation().truncate()).length() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((entity, mut bullet)) = bullets.get_mut(*e1) {
                handle_bullet(&mut commands, entity, &mut bullet);
                continue;
            }

            if let Ok((entity, mut bullet)) = bullets.get_mut(*e2) {
                handle_bullet(&mut commands, entity, &mut bullet);
            }
        }
    }
}

fn handle_bullet(commands: &mut Commands, entity: Entity, bullet: &mut Bullet) {
    info!("Bullet collided with something, piercing: {}", bullet.piercing);
    if bullet.piercing <= 0 {
        commands.entity(entity).despawn();
    } else {
        bullet.piercing -= 1;
    }
}
