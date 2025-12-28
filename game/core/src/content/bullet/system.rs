use std::f32::consts::PI;

use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy_rapier2d::na::Vector2;

use crate::common::damage::component::Damage;
use crate::common::damage::message::DamageEvent;
use crate::common::entity_type::EntityType;
use crate::common::lifetime::component::Lifetime;
use crate::constant::LAYER_BULLET;
use crate::content::block::component::{
    BurstShootPattern, BurstShootPatternShootingState, BurstShootPatternState, Health,
    LaserBeamPattern, LaserBeamPatternShootingState, LaserBeamPatternState, PullingBeamPattern,
    PullingBeamPatternState, ShootingProjectile, SingleShootPattern, SingleShootPatternState,
};
use crate::content::block::radar::component::Radar;
use crate::content::block::turret::component::{Turret, TurretState};
use crate::content::bullet::component::Laser;
use crate::content::ship::component::Ship;
use crate::content::team::Team;
use crate::{
    common::death::component::Death,
    content::bullet::component::{Bullet, Missile, MissileHitbox},
};
use bevy_rapier2d::parry::shape::Cuboid;
use bevy_rapier2d::prelude::*;

pub fn handle_bullets(
    mut query: Query<(&mut Bullet, &mut Transform), Without<Death>>,
    time: Res<Time>,
) {
    for (bullet, mut transform) in query.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_secs();
    }
}

pub fn destroy_out_of_range_bullet(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet, &GlobalTransform)>,
) {
    for (entity, bullet, global_transform) in query.iter_mut() {
        if bullet.range.squared()
            < global_transform
                .translation()
                .truncate()
                .distance_squared(bullet.shooted_at)
        {
            commands.entity(entity).insert(Death);
        }
    }
}

pub fn handle_missiles(
    mut missiles: Query<(&mut Missile, &mut Transform, &GlobalTransform), Without<Death>>,
    targets: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (mut missile, mut transform, global) in missiles.iter_mut() {
        let forward: Vec2 = global.up().truncate();

        missile.age += time.delta_secs();

        let t = (missile.age / missile.accel_time).clamp(0.4, 1.0);
        let eased = t.powf(1.2);

        let current_speed = missile.speed * eased;

        if let Some(target) = missile.target {
            if let Ok(target_gt) = targets.get(target) {
                let to_target =
                    target_gt.translation().truncate() - global.translation().truncate();

                if to_target.length_squared() < 0.0001 {
                    continue;
                }

                let target_dir = to_target.normalize();
                let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));

                let max_turn = 2.0 * time.delta_secs();
                let turn = angle_error.clamp(-max_turn, max_turn);

                transform.translation += (forward * current_speed * time.delta_secs()).extend(0.0);

                if angle_error.abs() < PI / 2.0 {
                    transform.rotate_z(turn);
                }

                continue;
            };
        };

        transform.translation += (forward * current_speed * time.delta_secs()).extend(0.0);
    }
}

pub fn handle_single_shot(
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

        if time.elapsed_secs() >= shoot_pattern_state.last_fired_at + shoot_pattern.fire_rate {
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

pub fn handle_burst_shot(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        &GlobalTransform,
        &Turret,
        &mut TurretState,
        &BurstShootPattern,
        &mut BurstShootPatternState,
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

        match &mut shoot_pattern_state.shooting {
            BurstShootPatternShootingState::Shooting {
                current_barrel,
                last_barrel_fired_at,
                current_amount,
            } => {
                if *current_amount <= 0 {
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Cooldown {
                        last_fired_at: time.elapsed_secs(),
                    };
                    continue;
                }

                if time.elapsed_secs() < *last_barrel_fired_at + shoot_pattern.delay {
                    continue;
                }

                let barrel_index = *current_barrel % barrels.len();
                let barrel_position = barrels[barrel_index];

                let bullet = spawn_bullet(
                    &mut commands,
                    transform,
                    team,
                    barrel_position,
                    *range,
                    &shoot_pattern.projectile,
                    turret_state.target,
                );

                if let Some(shoot_sound) = &shoot_pattern.shoot_sound {
                    commands.entity(bullet).insert((
                        AudioPlayer::new(shoot_sound.clone()),
                        PlaybackSettings::REMOVE.with_spatial(true),
                    ));
                }

                *last_barrel_fired_at = time.elapsed_secs();
                *current_amount -= 1;
                *current_barrel += 1;
            }
            BurstShootPatternShootingState::Cooldown { last_fired_at } => {
                if time.elapsed_secs() >= *last_fired_at + shoot_pattern.fire_rate {
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Shooting {
                        current_barrel: 0,
                        last_barrel_fired_at: 0.0,
                        current_amount: shoot_pattern.amount,
                    };
                }
            }
        }
    }
}

pub fn handle_laser_beam(
    time: Res<Time>,
    rapier: ReadRapierContext,
    healths: Query<(Entity, &Health)>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &Turret,
        &mut TurretState,
        &LaserBeamPattern,
        &mut LaserBeamPatternState,
        &Team,
    )>,
    mut events: MessageWriter<DamageEvent>,
) {
    let collision = rapier.single().expect("How to get rapier context?");

    for (entity, transform, turret, turret_state, shoot_pattern, mut shoot_pattern_state, team) in
        query.iter_mut()
    {
        let Turret { barrels, range, .. } = turret;

        match &mut shoot_pattern_state.shooting {
            LaserBeamPatternShootingState::Shooting { start_time } => {
                if time.elapsed_secs() >= *start_time + shoot_pattern.duration {
                    shoot_pattern_state.shooting = LaserBeamPatternShootingState::Cooldown {
                        last_fired_at: time.elapsed_secs(),
                    };

                    if let Some(beam) = shoot_pattern_state.beam {
                        commands.entity(beam).insert(Visibility::Hidden);
                    }

                    continue;
                }

                let intensitive = (time.elapsed_secs() - *start_time)
                    .min(1.0)
                    .min((*start_time + shoot_pattern.duration - time.elapsed_secs()).min(1.0))
                    * 10.0;

                if !turret_state.active {
                    continue;
                }

                let filter = QueryFilter::from(CollisionGroups::new(
                    team.bullet_group(),
                    team.bullet_filter(),
                ));

                for barrel in barrels {
                    let origin = transform.translation().truncate() + barrel + Vec2::new(0.0, 0.0);

                    let shape = &Cuboid::new(Vector2::new(shoot_pattern.width, *range));

                    collision.intersect_shape(
                        origin,
                        transform.rotation().to_euler(EulerRot::XYZ).2,
                        shape,
                        filter,
                        |entity| {
                            if let Ok((health, _)) = healths.get(entity) {
                                events.write(DamageEvent {
                                    attacker: entity,
                                    target: health,
                                    damage: shoot_pattern.damage,
                                });
                            }
                            true
                        },
                    );
                }

                if let Some(beam) = shoot_pattern_state.beam {
                    commands.entity(beam).insert((
                        Visibility::Visible,
                        Sprite {
                            color: team.color(intensitive),
                            custom_size: Some(Vec2::new(shoot_pattern.width, *range)),
                            ..default()
                        },
                    ));
                } else {
                    let beam = commands
                        .spawn((
                            Sprite {
                                color: team.color(intensitive),
                                custom_size: Some(Vec2::new(shoot_pattern.width, *range)),
                                ..default()
                            },
                            Transform::from_translation(
                                Vec2::new(0.0, *range / 2.0).extend(LAYER_BULLET),
                            ),
                            Visibility::Hidden,
                        ))
                        .id();

                    shoot_pattern_state.beam = Some(beam);
                    commands.entity(entity).add_child(beam);
                }
            }
            LaserBeamPatternShootingState::Cooldown { last_fired_at } => {
                if !turret_state.active {
                    continue;
                }

                if time.elapsed_secs() >= *last_fired_at + shoot_pattern.cooldown {
                    shoot_pattern_state.shooting = LaserBeamPatternShootingState::Shooting {
                        start_time: time.elapsed_secs(),
                    };
                }
            }
        };
    }
}

pub fn handle_pulling_beam(
    rapier: ReadRapierContext,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &Turret,
        &mut TurretState,
        &PullingBeamPattern,
        &mut PullingBeamPatternState,
        &Team,
        Option<&Radar>,
    )>,
    mut ships: Query<(&Ship, &mut ExternalForce)>,
) {
    let collision = rapier.single().expect("How to get rapier context?");

    for (
        entity,
        transform,
        turret,
        turret_state,
        shoot_pattern,
        mut shoot_pattern_state,
        team,
        radar,
    ) in query.iter_mut()
    {
        let Turret { barrels, range, .. } = turret;
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
) -> Entity {
    let rotation = turret_transform.rotation();
    let offset = rotation * barrel_position.extend(LAYER_BULLET);
    let spawn_position = turret_transform.translation() + offset;
    let dir = (rotation * Vec3::Y).truncate().normalize();

    match &projectile {
        ShootingProjectile::Bullet {
            speed,
            damage,
            lifetime,
            piercing,
            width,
            length,
        } => {
            let bullet = commands
                .spawn((
                    EntityType::Projectile,
                    Lifetime::seconds(*lifetime),
                    Bullet {
                        shooted_at: spawn_position.truncate(),
                        range,
                        piercing: *piercing,
                        velocity: dir * *speed,
                    },
                    Damage { damage: *damage },
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(*width, *length)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    Collider::ball(*width / 2.0),
                    CollisionGroups::new(team.bullet_group(), team.bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Ccd::enabled(),
                    Sensor,
                    *team,
                ))
                .id();

            if (*piercing) > 0 {
                commands.entity(bullet).insert(Sensor);
            }

            return bullet;
        }
        ShootingProjectile::Laser { damage, width, .. } => {
            let bullet = commands
                .spawn((
                    EntityType::Projectile,
                    Laser::Init {
                        width: *width,
                        length: range,
                        origin: spawn_position.truncate(),
                        rotation,
                    },
                    Damage { damage: *damage },
                    *team,
                ))
                .id();

            return bullet;
        }
        ShootingProjectile::Missile {
            speed,
            damage,
            lifetime,
            lock_range,
            blast_damage,
            blast_radius,
            width,
            length,
        } => {
            let missile = commands
                .spawn((
                    EntityType::Projectile,
                    Lifetime::seconds(*lifetime),
                    Missile {
                        shooted_at: spawn_position.truncate(),
                        lock_range: *lock_range,
                        age: 0.0,
                        speed: *speed,
                        accel_time: 0.35,
                        target,
                    },
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(*width, *length)),
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

            if target.is_none() {
                commands
                    .entity(missile)
                    .insert((Radar::new(*lock_range, 1.0),));
            }

            let hit_box = commands
                .spawn((
                    Collider::ball(*width / 2.0),
                    CollisionGroups::new(team.bullet_group(), team.bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    Damage { damage: *damage },
                    Transform::default(),
                    MissileHitbox {
                        missile,
                        blast_damage: *blast_damage,
                        blast_radius: *blast_radius,
                        team: *team,
                    },
                ))
                .id();

            commands.entity(missile).add_child(hit_box);

            return missile;
        }
    }
}

pub fn handle_laser(
    mut commands: Commands,
    mut events: MessageWriter<DamageEvent>,
    mut lasers: Query<(Entity, &Laser, &Damage, &Team, Option<&mut Sprite>)>,
    rapier: ReadRapierContext,
    time: Res<Time>,
) {
    for (entity, laser, damage, team, sprite) in lasers.iter_mut() {
        match laser {
            Laser::Init {
                length,
                width,
                origin,
                rotation,
            } => {
                let dir = (*rotation * Vec3::Y).truncate().normalize();

                let mut laser_length = *length;

                let collision = rapier.single().expect("How to get rapier context?");
                let filter = QueryFilter::from(CollisionGroups::new(
                    team.bullet_group(),
                    team.bullet_filter(),
                ));

                if let Some((target, toi)) =
                    collision.cast_ray(*origin, dir, *length, false, filter)
                {
                    laser_length = toi;
                    events.write(DamageEvent {
                        attacker: entity,
                        target: target,
                        damage: damage.damage,
                    });
                }

                commands.entity(entity).insert((
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(*width, laser_length)),
                        ..default()
                    },
                    Transform {
                        translation: origin.extend(LAYER_BULLET)
                            + *rotation
                                * Vec2::new(*width / 2.0, laser_length / 2.0).extend(LAYER_BULLET),
                        rotation: *rotation,
                        ..default()
                    },
                ));

                commands.entity(entity).insert(Laser::Fired {
                    fired_at: time.elapsed_secs(),
                });
            }
            Laser::Fired { fired_at } => {
                const FADE_TIME: f32 = 0.25;

                if time.elapsed_secs() - *fired_at > FADE_TIME {
                    commands.entity(entity).insert(Death);
                } else {
                    let intensive = 10.0 * (FADE_TIME - (time.elapsed_secs() - *fired_at));
                    if let Some(mut sprite) = sprite {
                        sprite.color = team.color(intensive);
                    }
                }
            }
        }
    }
}

pub fn handle_missile_radar_target(
    mut missile_radars: Query<(&mut Missile, &Radar), Without<Death>>,
) {
    'outer: for (mut missile, radar) in missile_radars.iter_mut() {
        if let Some(current) = missile.target {
            if radar.targets.is_empty() {
                missile.target = None;
                continue 'outer;
            }

            for target in radar.targets.iter() {
                if current == *target {
                    continue 'outer;
                }
            }
            missile.target = Some(*radar.targets.first().expect("How does this event happend"));
        } else {
            if let Some(target) = radar.targets.first() {
                missile.target = Some(*target);
            }
        }
    }
}

pub fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<EntityType>>) {
    for entity in &bullets {
        commands.entity(entity).insert(Death);
    }
}
