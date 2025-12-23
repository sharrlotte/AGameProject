use bevy::math::FloatPow;
use bevy::prelude::*;

use crate::components::damage::Damage;
use crate::components::entity_type::EntityType;
use crate::content::block::base::{
    BurstShootPattern, BurstShootPatternState, ShootingProjectile, SingleShootPattern,
    SingleShootPatternState,
};
use crate::content::block::radar::Radar;
use crate::content::bullet::base::Lifetime;
use crate::content::team::Team;
use crate::content::turret::base::{Turret, TurretState};
use crate::{
    components::death::Death,
    content::bullet::base::{Bullet, Missile, MissileHitbox},
};
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
                let max_turn = 1.5 * time.delta_secs();
                let turn = angle_error.clamp(-max_turn, max_turn);

                transform.rotate_z(turn);
                transform.translation += (forward * current_speed * time.delta_secs()).extend(0.0);

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

        if time.elapsed_secs() >= shoot_pattern_state.last_fired_at + shoot_pattern.fire_rate {
            shoot_pattern_state.is_bursting = true;
            shoot_pattern_state.current_amount = shoot_pattern.amount;
            shoot_pattern_state.last_fired_at = time.elapsed_secs();
        } else if !shoot_pattern_state.is_bursting {
            continue;
        }

        if time.elapsed_secs() < shoot_pattern_state.last_barrel_fired_at + shoot_pattern.delay {
            continue;
        }

        let barrel_index = shoot_pattern_state.current_barrel % barrels.len();
        let barrel_position = barrels[barrel_index];

        if shoot_pattern_state.current_amount <= 0 {
            shoot_pattern_state.is_bursting = false;
            continue;
        }

        spawn_bullet(
            &mut commands,
            transform,
            team,
            barrel_position,
            *range,
            &shoot_pattern.projectile,
            turret_state.target,
        );

        shoot_pattern_state.last_barrel_fired_at = time.elapsed_secs();
        shoot_pattern_state.current_amount -= 1;
        shoot_pattern_state.current_barrel += 1;
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
                    Lifetime::new(*lifetime),
                    Bullet {
                        shooted_at: spawn_position.truncate(),
                        range,
                        piercing: *piercing,
                        velocity: dir * *speed,
                    },
                    Damage { damage: *damage },
                    Sprite {
                        color: team.color(5.0),
                        custom_size: Some(Vec2::new(2.0, 50.0)),
                        ..default()
                    },
                    Transform {
                        translation: spawn_position,
                        rotation,
                        ..default()
                    },
                    Collider::ball(2.0),
                    CollisionGroups::new(team.bullet_group(), team.bullet_filter()),
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
            blast_damage,
            blast_radius,
        } => {
            let missile = commands
                .spawn((
                    EntityType::Projectile,
                    Lifetime::new(*lifetime),
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
                        custom_size: Some(Vec2::new(4.0, 50.0)),
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
                    Collider::ball(5.0),
                    CollisionGroups::new(team.bullet_group(), team.bullet_filter()),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    Damage { damage: *damage },
                    MissileHitbox {
                        missile,
                        blast_damage: *blast_damage,
                        blast_radius: *blast_radius,
                        team: *team,
                    },
                    Transform::default(),
                ))
                .id();

            commands.entity(missile).add_child(hit_box);
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
