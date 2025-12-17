use crate::content::ships::features::shooting::Bullet;
use crate::content::team::Team;
use crate::content::turret::{EmissionType, FireIntent, FireStyle, TurretBarrel, TurretEmission};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*; // Reusing existing Bullet component if possible, or define new one.

pub fn handle_turret_emission(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &FireIntent,
        &mut TurretEmission,
        &mut TurretBarrel,
        &mut FireStyle,
        Option<&Team>,
    )>,
) {
    for (entity, transform, intent, mut emission, mut barrel, mut style, team) in query.iter_mut() {
        emission.timer += time.delta_secs();

        if let Some(target_pos) = intent.target_pos {
            match *style {
                FireStyle::Single => {
                    if intent.active && emission.timer >= emission.fire_rate {
                        fire_projectiles(
                            &mut commands,
                            entity,
                            transform,
                            &mut barrel,
                            &emission,
                            team,
                            target_pos,
                        );
                        emission.timer = 0.0;
                    }
                }
                FireStyle::Burst {
                    count,
                    delay,
                    ref mut current_shot,
                    ref mut burst_timer,
                    ref mut is_bursting,
                } => {
                    if !*is_bursting && intent.active && emission.timer >= emission.fire_rate {
                        *is_bursting = true;
                        *current_shot = 0;
                        *burst_timer = delay; // Start immediately? Or wait delay? Usually immediate first shot.
                        // Let's say immediate first shot.
                        fire_projectiles(
                            &mut commands,
                            entity,
                            transform,
                            &mut barrel,
                            &emission,
                            team,
                            target_pos,
                        );
                        *current_shot += 1;
                        emission.timer = 0.0;
                    } else if *is_bursting {
                        *burst_timer += time.delta_secs();
                        if *burst_timer >= delay {
                            fire_projectiles(
                                &mut commands,
                                entity,
                                transform,
                                &mut barrel,
                                &emission,
                                team,
                                target_pos,
                            );
                            *current_shot += 1;
                            *burst_timer = 0.0;

                            if *current_shot >= count {
                                *is_bursting = false;
                            }
                        }
                    }
                }
                FireStyle::Continuous => {
                    if intent.active {
                        match emission.emission_type {
                            EmissionType::Bullet { .. } => {
                                if emission.timer >= emission.fire_rate {
                                    fire_projectiles(
                                        &mut commands,
                                        entity,
                                        transform,
                                        &mut barrel,
                                        &emission,
                                        team,
                                        target_pos,
                                    );
                                    emission.timer = 0.0;
                                }
                            }
                            EmissionType::Beam { .. } => {
                                // Beam logic would go here (e.g. raycasting)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fire_projectiles(
    commands: &mut Commands,
    _turret_entity: Entity,
    turret_transform: &GlobalTransform,
    barrel: &mut TurretBarrel,
    emission: &TurretEmission,
    team: Option<&Team>,
    target_pos: Vec2,
) {
    let turret_pos = turret_transform.translation().truncate();
    let dir_to_target = (target_pos - turret_pos).normalize_or_zero();
    let angle = Vec2::Y.angle_to(dir_to_target);
    let rotation = Quat::from_rotation_z(angle);

    let barrels_to_fire = if barrel.fire_all_at_once {
        0..barrel.barrel_offsets.len()
    } else {
        if barrel.barrel_offsets.is_empty() {
            0..0
        } else {
            let idx = barrel.current_barrel_index;
            barrel.current_barrel_index =
                (barrel.current_barrel_index + 1) % barrel.barrel_offsets.len();
            idx..idx + 1
        }
    };

    let team_comp = team.copied().unwrap_or_default();

    for i in barrels_to_fire {
        let offset = barrel.barrel_offsets.get(i).cloned().unwrap_or(Vec2::ZERO);
        // Transform offset by turret rotation
        // We need the turret's current rotation from GlobalTransform
        let (_, turret_rot, _) = turret_transform.to_scale_rotation_translation();
        let world_offset = turret_rot * offset.extend(0.0);
        let spawn_pos = turret_transform.translation() + world_offset;

        match &emission.emission_type {
            EmissionType::Bullet {
                speed,
                damage: _,
                lifetime,
            } => {
                commands.spawn((
                    Bullet {
                        timer: Timer::from_seconds(*lifetime, TimerMode::Once),
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
                    Collider::ball(2.0),
                    Sensor,
                    Velocity::linear(dir_to_target * *speed),
                    ActiveEvents::COLLISION_EVENTS,
                    GravityScale(0.0),
                    CollisionGroups::new(Group::default(), team_comp.get_bullet_filter()),
                    team_comp,
                ));
            }
            EmissionType::Beam { .. } => {
                // Beam implementation omitted for brevity
            }
        }
    }
}
