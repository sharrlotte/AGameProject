use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy_rapier2d::parry::shape::Cuboid;
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;

use super::*;
use crate::config::GameConfigValue;
use crate::constant::{GRID_SIZE, LAYER_BULLET};
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::power::{Boost, STARVATION_THRESHOLD, Supply};
use crate::content::block::feature::turret::{
    Turret, TurretBarrel, TurretCharge, TurretPower, TurretSprite, TurretState, TurretTarget,
};
use crate::content::block::radar::Radar;
use crate::content::block::{
    BlockOwner, BurstShootPattern, BurstShootPatternShootingState, BurstShootPatternState, Health,
    LaserBeamPattern, LaserBeamPatternShootingState, LaserBeamPatternState, PullingBeamPattern,
    PullingBeamPatternState, ShootingProjectile, SingleShootPattern, SingleShootPatternState,
};
use crate::content::damage::{Damage, DamageEvent};
use crate::content::lifetime::Lifetime;
use crate::content::body::BlockGrid;
use crate::content::ship::Ship;
use crate::content::team::Team;
use crate::resource::SceneState;


pub fn destroy_out_of_range_bullet(
    mut commands: Commands,
    query: Query<(Entity, &Bullet, &Transform)>,
) {
    for (entity, bullet, transform) in query.iter() {
        if bullet.range.squared()
            < transform
                .translation
                .truncate()
                .distance_squared(bullet.shooted_at)
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_missiles(
    mut missiles: Query<(&mut Missile, &mut Transform, &GlobalTransform)>,
    targets: Query<(&GlobalTransform, &BlockGrid)>,
    time: Res<Time>,
    config: Res<GameConfigValue>,
) {
    for (mut missile, mut transform, global) in missiles.iter_mut() {
        let forward: Vec2 = global.up().truncate();

        missile.age += time.delta_secs();

        let t = (missile.age / missile.accel_time).clamp(0.4, 1.0);
        let eased = t.powf(1.2);

        let current_speed = missile.speed * eased;

        if let Some(target) = missile.target {
            if let Ok((target_gt, target_blocks)) = targets.get(target) {
                let Some(target_block) = target_blocks.blocks().next() else {
                    continue;
                };

                let to_target = target_gt
                    .transform_point(
                        (target_block.placement.position_pixel() + GRID_SIZE / 2.0).extend(0.0),
                    )
                    .truncate()
                    - global.translation().truncate();

                if to_target.length_squared() < 0.0001 {
                    continue;
                }

                let target_dir = to_target.normalize_or_zero();
                let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));

                let max_turn = config.0.missile_max_turn * time.delta_secs();
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
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    mut query: Query<(
        &Turret,
        &mut TurretState,
        &SingleShootPattern,
        &mut SingleShootPatternState,
        &Team,
        Option<&mut AmmoState>,
        &TurretBarrel,
        Option<(&TurretPower, &mut TurretCharge)>,
        Option<&Boost>,
    )>,
    config: Res<GameConfigValue>,
) {
    let dt = time.delta_secs();
    for (
        turret,
        turret_state,
        shoot_pattern,
        mut shoot_pattern_state,
        team,
        mut ammo_state,
        barrel_ref,
        mut turret_power,
        boost,
    ) in query.iter_mut()
    {
        shoot_pattern_state.cooldown_remaining =
            (shoot_pattern_state.cooldown_remaining - dt).max(0.0);

        if !turret_state.active || shoot_pattern_state.cooldown_remaining > 0.0 {
            continue;
        }

        let Ok(transform) = barrels.get(barrel_ref.0) else {
            continue;
        };
        let Turret { barrels, range, .. } = turret;

        let boost_val = boost.map_or(1.0, Boost::value).max(1.0);
        let cooldown = shoot_pattern.fire_rate / boost_val;

        if let Some((TurretPower::Charge { capacity, .. }, ref charge)) = turret_power
            && charge.value < *capacity - 1e-5
        {
            continue;
        }

        for barrel_position in barrels.iter() {
            if let Some(ref mut ammo) = ammo_state
                && !ammo.take()
            {
                continue;
            }

            spawn_bullet(
                &mut commands,
                transform,
                team,
                *barrel_position,
                *range,
                &shoot_pattern.projectile,
                match turret_state.target {
                    Some(TurretTarget::Ship(entity)) => Some(entity),
                    _ => None,
                },
                &config,
            );
        }
        shoot_pattern_state.cooldown_remaining = cooldown;
        if let Some((TurretPower::Charge { .. }, ref mut charge)) = turret_power {
            charge.value = 0.0;
        }
    }
}

pub fn handle_burst_shot(
    time: Res<Time>,
    mut commands: Commands,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    mut query: Query<(
        &Turret,
        &mut TurretState,
        &BurstShootPattern,
        &mut BurstShootPatternState,
        &Team,
        Option<&mut AmmoState>,
        &TurretBarrel,
        Option<(&TurretPower, &mut TurretCharge)>,
        Option<&Boost>,
    )>,
    config: Res<GameConfigValue>,
) {
    let dt = time.delta_secs();
    for (
        turret,
        turret_state,
        shoot_pattern,
        mut shoot_pattern_state,
        team,
        mut ammo_state,
        barrel_ref,
        mut turret_power,
        boost,
    ) in query.iter_mut()
    {
        let boost_val = boost.map_or(1.0, Boost::value).max(1.0);
        let cooldown = shoot_pattern.fire_rate / boost_val;

        match &mut shoot_pattern_state.shooting {
            BurstShootPatternShootingState::Shooting {
                current_barrel,
                barrel_delay_remaining,
                current_amount,
            } => {
                *barrel_delay_remaining = (*barrel_delay_remaining - dt).max(0.0);

                if !turret_state.active {
                    continue;
                }

                let Ok(transform) = barrels.get(barrel_ref.0) else {
                    continue;
                };
                let Turret { barrels, range, .. } = turret;

                if *current_amount <= 0 {
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Cooldown {
                        cooldown_remaining: cooldown,
                    };
                    continue;
                }

                if *barrel_delay_remaining > 0.0 {
                    continue;
                }

                if barrels.is_empty() {
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Cooldown {
                        cooldown_remaining: cooldown,
                    };
                    continue;
                }
                let barrel_index = *current_barrel % barrels.len();
                let barrel_position = barrels[barrel_index];

                if let Some(ref mut ammo) = ammo_state
                    && !ammo.take()
                {
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Cooldown {
                        cooldown_remaining: cooldown,
                    };
                    continue;
                }

                let bullet = spawn_bullet(
                    &mut commands,
                    transform,
                    team,
                    barrel_position,
                    *range,
                    &shoot_pattern.projectile,
                    match turret_state.target {
                        Some(TurretTarget::Ship(entity)) => Some(entity),
                        _ => None,
                    },
                    &config,
                );

                if let Some(shoot_sound) = &shoot_pattern.shoot_sound {
                    commands.entity(bullet).insert((
                        AudioPlayer::new(shoot_sound.clone()),
                        PlaybackSettings::REMOVE.with_spatial(true),
                    ));
                }

                *barrel_delay_remaining = shoot_pattern.delay;
                *current_amount -= 1;
                *current_barrel += 1;
            }
            BurstShootPatternShootingState::Cooldown { cooldown_remaining } => {
                *cooldown_remaining = (*cooldown_remaining - dt).max(0.0);
                if !turret_state.active {
                    continue;
                }
                if *cooldown_remaining <= 0.0 {
                    if let Some((TurretPower::Charge { capacity, .. }, ref mut charge)) =
                        turret_power
                    {
                        if charge.value < *capacity - 1e-5 {
                            continue;
                        }
                        charge.value = 0.0;
                    }
                    shoot_pattern_state.shooting = BurstShootPatternShootingState::Shooting {
                        current_barrel: 0,
                        barrel_delay_remaining: 0.0,
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
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    mut query: Query<(
        Entity,
        &Turret,
        &mut TurretState,
        &LaserBeamPattern,
        &mut LaserBeamPatternState,
        &Team,
        &TurretBarrel,
        Option<&Supply>,
        Option<&Boost>,
    )>,
    mut events: MessageWriter<DamageEvent>,
) {
    let Ok(collision) = rapier.single() else {
        return;
    };

    let dt = time.delta_secs();
    for (
        _entity,
        turret,
        turret_state,
        shoot_pattern,
        mut shoot_pattern_state,
        team,
        barrel_ref,
        supply,
        boost,
    ) in query.iter_mut()
    {
        let supply_val = supply.map_or(1.0, Supply::value);
        let boost_val = boost.map_or(1.0, Boost::value).max(1.0);

        let Ok(transform) = barrels.get(barrel_ref.0) else {
            continue;
        };
        let Turret { barrels, range, .. } = turret;

        match &mut shoot_pattern_state.shooting {
            LaserBeamPatternShootingState::Shooting { elapsed } => {
                *elapsed += dt;
                if supply_val <= STARVATION_THRESHOLD || *elapsed >= shoot_pattern.duration {
                    shoot_pattern_state.shooting = LaserBeamPatternShootingState::Cooldown {
                        cooldown_remaining: shoot_pattern.cooldown / boost_val,
                    };

                    if let Some(beam) = shoot_pattern_state.beam {
                        commands.entity(beam).insert(Visibility::Hidden);
                    }

                    continue;
                }

                let intensitive = ((*elapsed)
                    .min(1.0)
                    .min((shoot_pattern.duration - *elapsed).min(1.0))
                    * 10.0)
                    .max(0.5);

                if !turret_state.active {
                    continue;
                }

                let filter = QueryFilter::from(CollisionGroups::new(
                    team.bullet_group(),
                    team.bullet_filter(),
                ));

                for barrel in barrels {
                    let origin = transform.translation().truncate() + barrel + Vec2::new(0.0, 0.0);

                    let shape = &Cuboid::new(Vec2::new(shoot_pattern.width, *range));

                    collision.intersect_shape(
                        origin,
                        transform.rotation().to_euler(EulerRot::XYZ).2,
                        shape,
                        filter,
                        |entity, _collider| {
                            if let Ok((health, _)) = healths.get(entity) {
                                events.write(DamageEvent {
                                    attacker: entity,
                                    target: health,
                                    damage: shoot_pattern.damage * supply_val,
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
                    // Parent to the rotating barrel so the visual direction
                    // always matches the raycast hitbox direction.
                    commands.entity(barrel_ref.0).add_child(beam);
                }
            }
            LaserBeamPatternShootingState::Cooldown { cooldown_remaining } => {
                *cooldown_remaining = (*cooldown_remaining - dt).max(0.0);
                if !turret_state.active || supply_val <= STARVATION_THRESHOLD {
                    continue;
                }

                if *cooldown_remaining <= 0.0 {
                    shoot_pattern_state.shooting =
                        LaserBeamPatternShootingState::Shooting { elapsed: 0.0 };
                }
            }
        };
    }
}

pub fn handle_pulling_beam(
    rapier: ReadRapierContext,
    mut commands: Commands,
    time: Res<Time>,
    block_owners: Query<&BlockOwner>,
    mut ships: Query<(&GlobalTransform, &mut Velocity), With<Ship>>,
    barrels: Query<&GlobalTransform, With<TurretSprite>>,
    mut query: Query<(
        Entity,
        &Turret,
        &TurretState,
        &PullingBeamPattern,
        &mut PullingBeamPatternState,
        &Team,
        &TurretBarrel,
        Option<&Supply>,
    )>,
) {
    const PULL_SPEED: f32 = 500.0;

    let Ok(collision) = rapier.single() else {
        return;
    };

    for (
        entity,
        turret,
        turret_state,
        shoot_pattern,
        mut state,
        team,
        barrel_ref,
        supply,
    ) in query.iter_mut()
    {
        let supply_val = supply.map_or(1.0, Supply::value);
        let Ok(transform) = barrels.get(barrel_ref.0) else {
            continue;
        };
        let Turret { barrels, range, .. } = turret;

        // Each target ship is pulled at most once per tick, even when several
        // barrel boxes overlap the same block.
        let mut pulled = HashSet::new();

        if !turret_state.active || supply_val <= STARVATION_THRESHOLD {
            if let Some(beam) = state.beam {
                commands.entity(beam).insert(Visibility::Hidden);
            }
            continue;
        }

        let filter = QueryFilter::from(CollisionGroups::new(
            team.bullet_group(),
            team.bullet_filter(),
        ));

        for barrel in barrels {
            let origin = transform.translation().truncate() + *barrel;
            let shape = &Cuboid::new(Vec2::new(shoot_pattern.width, *range));

            collision.intersect_shape(
                origin,
                transform.rotation().to_euler(EulerRot::XYZ).2,
                shape,
                filter,
                |block_entity, _| {
                    let Ok(owner) = block_owners.get(block_entity) else {
                        return true;
                    };

                    if owner.body == entity {
                        return true;
                    }

                    if !pulled.insert(owner.body) {
                        return true;
                    }

                    if let Ok((ship_transform, mut velocity)) = ships.get_mut(owner.body) {
                        let direction =
                            (origin - ship_transform.translation().truncate()).normalize_or_zero();
                            
                        velocity.linear += direction * PULL_SPEED * supply_val * time.delta_secs();
                    }

                    true
                },
            );
        }

        let beam_sprite = Sprite {
            color: team.color(2.0),
            custom_size: Some(Vec2::new(shoot_pattern.width, *range)),
            ..default()
        };

        if let Some(beam) = state.beam {
            commands
                .entity(beam)
                .insert((Visibility::Visible, beam_sprite));
        } else {
            let beam = commands
                .spawn((
                    beam_sprite,
                    Transform::from_translation(Vec2::new(0.0, *range / 2.0).extend(LAYER_BULLET)),
                    Visibility::Hidden,
                ))
                .id();

            state.beam = Some(beam);
            commands.entity(barrel_ref.0).add_child(beam);
        }
    }
}

pub fn handle_point_defense_hits(
    mut collision_events: MessageReader<CollisionEvent>,
    owners: Query<&ProjectileOwner>,
    interceptables: Query<Entity, With<Interceptable>>,
    mut commands: Commands,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(first, second, _) = event else {
            continue;
        };
        for (interceptor, target) in [(*first, *second), (*second, *first)] {
            let (Ok(interceptor_owner), Ok(target_owner)) =
                (owners.get(interceptor), owners.get(target))
            else {
                continue;
            };
            if interceptor_owner.team == target_owner.team {
                continue;
            }
            if interceptables.contains(target) {
                commands.entity(target).despawn();
                commands.entity(interceptor).despawn();
            }
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
    config: &GameConfigValue,
) -> Entity {
    let rotation = turret_transform.rotation();
    let offset = rotation * barrel_position.extend(LAYER_BULLET);
    let spawn_position = turret_transform.translation() + offset;
    let dir = (rotation * Vec3::Y).truncate().normalize_or_zero();
    let transform = Transform {
        translation: spawn_position,
        rotation,
        ..default()
    };

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
                    DespawnOnExit(SceneState::Game),
                    Lifetime::seconds(*lifetime),
                    Bullet {
                        shooted_at: spawn_position.truncate(),
                        range,
                        piercing: *piercing,
                    },
                    Damage { damage: *damage },
                    ProjectileOwner { team: team.id },
                    Interceptable,
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(*width, *length)),
                        ..default()
                    },
                    transform,
                    GlobalTransform::from(transform),
                ))
                .insert((
                    RigidBody::KinematicVelocityBased,
                    Velocity::linear(dir * *speed),
                    Collider::cuboid(*width / 2.0, *length / 2.0),
                    CollisionGroups::new(team.bullet_group(), team.bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Ccd::enabled(),
                    Sensor,
                    *team,
                ))
                .id();

            return bullet;
        }
        ShootingProjectile::Laser { damage, width, .. } => {
            let bullet = commands
                .spawn((
                    DespawnOnExit(SceneState::Game),
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
                    DespawnOnExit(SceneState::Game),
                    Lifetime::seconds(*lifetime),
                    ProjectileOwner { team: team.id },
                    Interceptable,
                    Missile {
                        shooted_at: spawn_position.truncate(),
                        lock_range: *lock_range,
                        age: 0.0,
                        speed: *speed,
                        accel_time: config.0.missile_accel_time,
                        target,
                    },
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(*width, *length)),
                        ..default()
                    },
                    transform,
                    GlobalTransform::from(transform),
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
                    Collider::cuboid(*width / 2.0, *length / 2.0),
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
    config: Res<GameConfigValue>,
) {
    for (entity, laser, damage, team, sprite) in lasers.iter_mut() {
        match laser {
            Laser::Init {
                length,
                width,
                origin,
                rotation,
            } => {
                let dir = (*rotation * Vec3::Y).truncate().normalize_or_zero();

                let mut laser_length = *length;

                let Ok(collision) = rapier.single() else {
                    return;
                };
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
                            + *rotation * Vec2::new(0.0, laser_length / 2.0).extend(LAYER_BULLET),
                        rotation: *rotation,
                        ..default()
                    },
                ));

                commands.entity(entity).insert(Laser::Fired {
                    fired_at: time.elapsed_secs(),
                });
            }
            Laser::Fired { fired_at } => {
                let fade_time = config.0.bullet_fade_time;

                if time.elapsed_secs() - *fired_at > fade_time {
                    commands.entity(entity).despawn();
                } else {
                    let intensive = 10.0 * (fade_time - (time.elapsed_secs() - *fired_at)).max(0.0);

                    if let Some(mut sprite) = sprite {
                        sprite.color = team.color(intensive);
                    }
                }
            }
        }
    }
}

pub fn handle_missile_radar_target(mut missile_radars: Query<(&mut Missile, &Radar)>) {
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

            if let Some(target) = radar.targets.first() {
                missile.target = Some(*target);
            }
        } else {
            if let Some(target) = radar.targets.first() {
                missile.target = Some(*target);
            }
        }
    }
}

pub fn handle_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut bullets: Query<(Entity, &mut Bullet)>,
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
        commands.entity(bullet).despawn();
    } else {
        bulletc.piercing -= 1;
    }
}

fn handle_missile_collision(
    commands: &mut Commands,
    hitbox: &MissileHitbox,
    transform: &GlobalTransform,
) {
    commands.entity(hitbox.missile).despawn();

    let transform = Transform::from(*transform);

    commands.spawn((
        hitbox.team,
        transform,
        GlobalTransform::from(transform),
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
