use crate::common::death::component::Death;
use crate::constant::{MAX_ZOOM, MIN_ZOOM};
use crate::content::block::component::BlockType;
use crate::content::block::message::BlockDestroyedEvent;
use crate::content::block::resource::{BlockRegistry, ShipCalculatedStat};
use crate::content::ship::component::{AIShipController, Ship, ShipBehavior, ShipUIRoot};
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::save::io::load_saved_schematics;
use crate::scene::game::resource::GameVars;
use crate::utils::random::random_vec2_step;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::Rng;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::f32::consts::PI;

use bevy::math::FloatPow;
use bevy_rapier2d::prelude::*;

use crate::content::ship::component::AIShipBehavior;

const SHIPS: usize = 20;

pub fn check_ship(
    mut commands: Commands,
    ships: Query<(&Ship, &Team)>,
    registry: Res<BlockRegistry>,
) {
    if ships.iter().len() > SHIPS {
        return;
    }

    let mut rng = rand::rng();
    let schematics = load_saved_schematics();

    let mut maps: HashMap<u8, u8> = HashMap::new();

    for i in 0..5 {
        maps.insert(i, 0);
    }

    for (_, team) in ships {
        if let Some(value) = maps.get(&team.id) {
            maps.insert(team.id, value + 1);
        } else {
            maps.insert(team.id, 1);
        }
    }

    let mut least_team_id = DEFAULT_TEAM_ID;
    let mut least_count: u8 = SHIPS as u8;

    for (team_id, count) in maps.iter() {
        if *count < least_count {
            least_team_id = *team_id;
            least_count = *count;
        }
    }

    let Some(schematic) = schematics.choose(&mut rng) else {
        return;
    };

    let position = random_vec2_step(
        Vec2::new(-50000.0, -50000.0),
        Vec2::new(50000.0, 50000.0),
        5000.0,
    );

    schematic.to_ship_data().spawn(
        &mut commands,
        &registry,
        Team::new(least_team_id),
        Transform::from_translation(position.extend(0.0)),
    );
}

pub fn cleanup_ships(mut commands: Commands, ships: Query<Entity, With<Ship>>) {
    for entity in &ships {
        commands.entity(entity).insert(Death);
    }
}

pub fn check_rebuild_ship_collider(
    mut commands: Commands,
    mut events: MessageReader<BlockDestroyedEvent>,
    mut ships: Query<(Entity, &mut Ship, &Team)>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((entity, mut ship, _)) = ships.get_mut(event.ship_id) else {
            continue;
        };

        if let Some(block) = registry.get(&event.block) {
            if block.block_type == BlockType::Core {
                commands.entity(entity).insert(Death);
                continue;
            }
        }

        ship.blocks.retain_mut(|f| f.id != event.block_id);

        let ShipCalculatedStat {
            mass,
            center,
            collider,
            thrust_power,
            turn_speed,
            size,
            core_position,
            min_range,
            max_range,
        } = registry.blocks_stats(&ship.blocks);

        commands.entity(entity).remove::<Collider>().insert((
            collider,
            ColliderMassProperties::Mass(mass),
            Ship {
                id: ship.id,
                blocks: ship.blocks.clone(),
                core_position,
                center,
                thrust_power,
                turn_speed,
                size,
                min_range,
                max_range,
            },
        ));
    }
}

pub fn update_ship_ui(
    mut commands: Commands,
    mut ui_roots: Query<(Entity, &ShipUIRoot, &mut Transform)>,
    ships: Query<(
        Entity,
        &Ship,
        &AIShipController,
        Option<&ShipBehavior>,
        &GlobalTransform,
    )>,
) {
    for (ui_entity, ui_root, mut transform) in ui_roots.iter_mut() {
        match ships.get(ui_root.ship) {
            Ok((ship, _, controller, behavior, global_transform)) => {
                transform.rotation = global_transform.rotation().inverse();

                let current = controller.current();

                commands.entity(ui_entity).insert((
                    Text2d::new(format!("Ship: {ship}\nC: {:?}\nB: {:?}", current, behavior)),
                    TextFont {
                        font_size: 90.0,
                        ..default()
                    },
                    Transform::default(),
                ));
            }
            Err(_) => {}
        }
    }
}

pub fn player_control_ship(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut ships: Query<(&mut ExternalForce, &Transform, &Ship), Without<Camera2d>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut state: ResMut<GameVars>,
) {
    for event in mouse_wheel_events.read() {
        if let Ok((_, mut projection)) = camera.single_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(MIN_ZOOM, MAX_ZOOM);
            }
        }
    }

    if let Some(following) = state.following {
        match ships.get(following) {
            Ok((_, transform, ship)) => {
                let target = transform
                    .transform_point(ship.core_position.extend(0.0))
                    .truncate();

                if let Ok((mut camera_transform, _)) = camera.single_mut() {
                    let current = camera_transform.translation.truncate();
                    let smooth_speed = 10.0;

                    let t = 1.0 - (-smooth_speed * time.delta_secs()).exp();
                    let new_position = current.lerp(target, t);

                    camera_transform.translation.x = new_position.x;
                    camera_transform.translation.y = new_position.y;
                }
            }
            Err(_) => {
                state.following = None;
            }
        }
    } else {
        if mouse.just_pressed(MouseButton::Left) {
            mouse_motion_events.clear();
        }

        if mouse.pressed(MouseButton::Left)
            && let Ok((mut transform, projection)) = camera.single_mut()
        {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };

            let pan_speed: f32 = scale * 0.5;

            for event in mouse_motion_events.read() {
                transform.translation.x -= event.delta.x * pan_speed;
                transform.translation.y += event.delta.y * pan_speed;
            }
        }

        if let Ok((mut camera_transform, projection)) = camera.single_mut() {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };

            let boost = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                3.0
            } else {
                1.0
            };

            let pan_speed = 180.0 * scale * boost * time.delta_secs();

            if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
                camera_transform.translation.y += 1.0 * pan_speed;
            } else if keyboard_input.pressed(KeyCode::KeyS)
                || keyboard_input.pressed(KeyCode::ArrowDown)
            {
                camera_transform.translation.y -= 1.0 * pan_speed;
            }

            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                camera_transform.translation.x -= 1.0 * pan_speed;
            } else if keyboard_input.pressed(KeyCode::KeyD)
                || keyboard_input.pressed(KeyCode::ArrowRight)
            {
                camera_transform.translation.x += 1.0 * pan_speed;
            }
        }
    }

    if let Some(controlling) = state.controlling {
        match ships.get_mut(controlling) {
            Ok((mut force, transform, ship)) => {
                let thrust_power = ship.thrust_power;
                let turn_speed = ship.turn_speed;

                let mut linear_force = Vec2::ZERO;
                let mut torque = 0.0;

                let forward = transform.local_y().truncate(); // Assuming +Y is forward
                let right = transform.local_x().truncate();

                if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp)
                {
                    linear_force += forward * thrust_power;
                }

                if keyboard_input.pressed(KeyCode::KeyS)
                    || keyboard_input.pressed(KeyCode::ArrowDown)
                {
                    linear_force -= forward * thrust_power;
                }

                if keyboard_input.pressed(KeyCode::KeyA)
                    || keyboard_input.pressed(KeyCode::ArrowLeft)
                {
                    torque += turn_speed;
                }

                if keyboard_input.pressed(KeyCode::KeyD)
                    || keyboard_input.pressed(KeyCode::ArrowRight)
                {
                    torque -= turn_speed;
                }

                // Strafe
                if keyboard_input.pressed(KeyCode::KeyQ) {
                    linear_force -= right * thrust_power;
                }
                if keyboard_input.pressed(KeyCode::KeyE) {
                    linear_force += right * thrust_power;
                }

                force.force = linear_force;
                force.torque = torque;
            }
            Err(_) => {
                state.controlling = None;
            }
        }
    } else {
    }
}

pub fn control_ship(
    time: Res<Time>,
    ships: Query<
        (
            Entity,
            &mut AIShipController,
            &mut ExternalForce,
            &Transform,
            &Ship,
            &Team,
        ),
        Without<Camera2d>,
    >,
    ships2: Query<(Entity, &GlobalTransform, &Ship, &Team)>,
    global_transforms: Query<&Transform, Without<Camera2d>>,
    state: ResMut<GameVars>,
    mut commands: Commands,
) {
    let mut maps: HashMap<
        u8,
        (
            Vec<(Entity, &GlobalTransform, &Ship, &Team)>,
            Vec<(Entity, &GlobalTransform, &Ship, &Team)>,
        ),
    > = HashMap::new();

    let mut rng = rand::rng();

    for ship in ships2 {
        let value = maps.get_mut(&ship.3.id);

        if let Some((ally, enemy)) = value {
            if ship.3.id == DEFAULT_TEAM_ID {
                ally.push(ship);
            } else {
                enemy.push(ship);
            }
        } else {
            if ship.3.id == DEFAULT_TEAM_ID {
                maps.insert(ship.3.id, (vec![ship], Vec::new()));
            } else {
                maps.insert(ship.3.id, (Vec::new(), vec![ship]));
            }
        }
    }

    for (entity, mut controller, _, transform, ship, team) in ships {
        if let Some(following) = state.following
            && following == entity
        {
            continue;
        }

        if let Some(controlling) = state.controlling
            && controlling == entity
        {
            continue;
        }

        if !controller.timer.tick(time.delta()).just_finished() {
            continue;
        }

        let (ally_with_self, enemy) = maps.get(&team.id).expect("How this is possible");

        let ally = ally_with_self
            .iter()
            .filter(|other| other.0 != entity)
            .collect::<Vec<_>>();

        let in_range: Vec<_> = enemy
            .iter()
            .filter(|other| {
                other
                    .1
                    .translation()
                    .truncate()
                    .distance(transform.translation.truncate())
                    <= ship.max_range
            })
            .collect();

        let best_enemy = in_range.iter().min_by(|a, b| {
            a.1.translation()
                .truncate()
                .distance_squared(transform.translation.truncate())
                .partial_cmp(
                    &b.1.translation()
                        .truncate()
                        .distance_squared(transform.translation.truncate()),
                )
                .expect("Odering failed?")
        });

        match controller.current() {
            AIShipBehavior::Regroup => {
                if let Some(a) = ally.first() {
                    let is_in_range = transform
                        .translation
                        .truncate()
                        .distance(a.1.translation().truncate())
                        <= (a.2.size.length() * ship.size.length()) * 2.0;

                    if is_in_range {
                        controller.next(AIShipBehavior::Wander);
                    } else {
                        commands.entity(entity).insert(ShipBehavior::Approach {
                            position: a.1.translation().truncate() + a.2.center,
                            radius: (a.2.size.length() * ship.size.length()) * 2.0,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Idle);
                }
            }
            AIShipBehavior::Idle => {
                if let Some(best_enemy) = best_enemy {
                    controller.next(AIShipBehavior::Chase {
                        target: best_enemy.0,
                    });
                } else {
                    if let Some(a) = ally.first() {
                        controller.next(AIShipBehavior::Chase { target: a.0 });
                    }
                }
            }
            AIShipBehavior::Chase { target } => {
                let position = global_transforms
                    .get(target)
                    .map(|transform| transform.translation.truncate());

                if let Ok(position) = position {
                    if transform.translation.distance_squared(position.extend(0.0))
                        <= ship.min_range
                    {
                        commands.entity(entity).insert(ShipBehavior::Approach {
                            position: position,
                            radius: ship.min_range * 0.9,
                        });
                    } else {
                        commands.entity(entity).insert(ShipBehavior::GoAround {
                            position: position,
                            radius: ship.min_range * 0.9,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Regroup);
                }
            }
            AIShipBehavior::Wander => {
                if rng.random_bool(0.5) {
                    controller.next(AIShipBehavior::Idle);
                } else if rng.random_bool(0.5) {
                    if let Some(a) = ally.first() {
                        controller.next(AIShipBehavior::Guard {
                            target: a.0,
                            radius: (a.2.size.length() * ship.size.length()) * 2.0,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Patrol {
                        around_position: Vec2::ZERO,
                        radius: ship.size.length() * 10.0,
                    });
                }
            }
            AIShipBehavior::Attack { target_position } => {
                commands.entity(entity).insert(ShipBehavior::Approach {
                    position: target_position,
                    radius: ship.size.length(),
                });
            }
            AIShipBehavior::Flee { from } => {
                let position = global_transforms
                    .get(from)
                    .map(|transform| transform.translation.truncate());

                if let Ok(position) = position {
                    let direction =
                        (transform.translation.truncate() - position).normalize_or_zero();

                    let op_direction = direction * -1.0;
                    let op_position = transform.translation.truncate() + op_direction * 1000.0;

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: op_position,
                        radius: ship.size.length(),
                    });
                }
            }
            AIShipBehavior::Patrol {
                around_position,
                radius,
            } => {
                commands.entity(entity).insert(ShipBehavior::Approach {
                    position: around_position,
                    radius,
                });
            }
            AIShipBehavior::Guard { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius,
                    });
                } else {
                    controller.previous();
                }
            }
            AIShipBehavior::Follow { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius,
                    });
                } else {
                    controller.previous();
                }
            }
            AIShipBehavior::Kamikaze { target } => {
                // Kamikaze at the target
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius: ship.size.length(),
                    });
                } else {
                    controller.previous();
                }
            }
        }
    }
}

pub fn update_ship_behavior(
    mut ships: Query<(
        Entity,
        &Ship,
        &ShipBehavior,
        &Transform,
        &mut ExternalForce,
        &Velocity,
    )>,
    state: Res<GameVars>,
    mut gizmos: Gizmos,
) {
    for (entity, ship, behavior, transform, mut force, velocity) in ships.iter_mut() {
        if let Some(controlling) = state.controlling
            && controlling == entity
        {
            continue;
        }

        let Ship {
            thrust_power,
            turn_speed,
            center,
            ..
        } = ship;

        let current_postion = transform.transform_point(center.extend(0.0)).truncate();
        let forward = transform.local_y().truncate();

        match behavior {
            ShipBehavior::Approach { position, radius } => {
                let direction = (position - current_postion).normalize_or_zero();
                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = angle - transform.rotation.to_euler(EulerRot::XYZ).2;
                let distance = current_postion.distance_squared(*position);

                let force_multiplier = if distance > radius.squared() {
                    1.0
                } else {
                    (distance / radius.squared()).squared()
                };

                let mut torque: f32 = if angle_diff > 0.0 {
                    *turn_speed
                } else {
                    -*turn_speed
                };

                if velocity.angvel.abs() > PI / 4.0 {
                    torque *= PI / 4.0 / velocity.angvel.abs();
                }

                force.torque = torque;

                if angle_diff.abs() < PI / 2.0 {
                    if distance <= *radius {
                        force.force = Vec2::ZERO;
                    } else {
                        force.force = forward * thrust_power * force_multiplier;
                    }
                }

                gizmos.arrow(
                    current_postion.extend(0.0),
                    current_postion.extend(0.0) + direction.extend(0.0) * 500.0,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
            ShipBehavior::GoAround { position, radius } => {
                let in_range = current_postion.distance(*position) <= *radius;
                let direction = (position - current_postion).normalize_or_zero();
                let position_on_ring = current_postion + direction * radius;
                let mut next_position = position_on_ring;

                if in_range {
                    next_position = position_on_ring
                        + Vec2::new(direction.y, -direction.x) * radius / radius.sqrt();
                }

                let direction = (next_position - current_postion).normalize_or_zero();

                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = angle - transform.rotation.to_euler(EulerRot::XYZ).2;
                let distance = current_postion.distance(next_position);

                let force_multiplier = if distance > radius.squared() {
                    1.0
                } else {
                    (distance / radius.squared()).squared()
                };

                let mut torque: f32 = if angle_diff > 0.0 {
                    *turn_speed
                } else {
                    -*turn_speed
                };

                if velocity.angvel.abs() > PI / 4.0 {
                    torque *= PI / 4.0 / velocity.angvel.abs();
                }

                force.torque = torque;

                if angle_diff.abs() < PI / 2.0 {
                    if distance <= *radius {
                        force.force = Vec2::ZERO;
                    } else {
                        force.force = forward * thrust_power * force_multiplier;
                    }
                }
            }
        }
    }
}
