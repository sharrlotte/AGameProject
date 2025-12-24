use crate::common::death::component::Death;
use crate::constant::{THRUST_POWER_FACTOR, TURN_SPEED_FACTOR};
use crate::content::block::component::BlockType;
use crate::content::block::message::BlockDestroyedEvent;
use crate::content::block::resource::{BlockRegistry, ShipCalculatedStat};
use crate::content::ship::component::{AIShipController, Ship};
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

pub fn check_ship(mut commands: Commands, ships: Query<&Ship>, registry: Res<BlockRegistry>) {
    if ships.iter().len() > SHIPS {
        return;
    }

    let missing = SHIPS - ships.iter().len();
    let mut rng = rand::rng();
    let schematics = load_saved_schematics();

    for _ in 0..missing {
        let team_id = rng.random_range(0..5);

        let Some(schematic) = schematics.choose(&mut rng) else {
            continue;
        };

        let position = random_vec2_step(
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(10000.0, 10000.0),
            500.0,
        );

        schematic.to_ship_data().spawn(
            &mut commands,
            &registry,
            Team::new(team_id),
            Transform::from_translation(position.extend(0.0)),
        );
    }
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
                radar_range: 3000.0,
                size,
            },
        ));
    }
}

pub fn control_ship(
    time: Res<Time>,
    ships: Query<
        (
            Entity,
            &mut AIShipController,
            &mut ExternalForce,
            &Velocity,
            &Transform,
            &Ship,
            &Team,
        ),
        Without<Camera2d>,
    >,
    ships2: Query<(Entity, &GlobalTransform, &Ship, &Team)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    global_transforms: Query<&Transform, Without<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut state: ResMut<GameVars>,
) {
    for event in mouse_wheel_events.read() {
        for (_, mut projection) in camera.iter_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(0.1, 5.0);
            }
        }
    }

    let mut maps: HashMap<
        u8,
        (
            Vec<(Entity, &GlobalTransform, &Ship, &Team)>,
            Vec<(Entity, &GlobalTransform, &Ship, &Team)>,
        ),
    > = HashMap::new();

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

    if let Some(following) = state.following {
        if ships.get(following).is_err() {
            state.following = None;
        }
    }

    if let Some(controlling) = state.controlling {
        if ships.get(controlling).is_err() {
            state.controlling = None;
        }
    }

    for (entity, mut controller, mut force, velocity, transform, ship, team) in ships {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        if let Some(following) = state.following {
            if following == entity {
                let target = transform
                    .transform_point(ship.core_position.extend(0.0))
                    .truncate();

                for (mut camera_transform, _) in camera.iter_mut() {
                    let current = camera_transform.translation.truncate();

                    let smooth_speed = 10.0;
                    let t = 1.0 - (-smooth_speed * time.delta_secs());
                    let new_position = current.lerp(target, t);

                    camera_transform.translation.x = new_position.x;
                    camera_transform.translation.y = new_position.y;
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
        }

        if let Some(controlling) = state.controlling {
            if controlling == entity {
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

                continue;
            }
        }

        if !controller.timer.tick(time.delta()).just_finished() {
            continue;
        }

        let (ally, enemy) = maps.get(&team.id).expect("How this is possible");

        let in_range: Vec<_> = enemy
            .iter()
            .filter(|other| {
                other
                    .1
                    .translation()
                    .truncate()
                    .distance(transform.translation.truncate())
                    <= ship.radar_range
            })
            .collect();

        match controller.current() {
            AIShipBehavior::Idle => {
                if !in_range.is_empty() {
                    controller.next(AIShipBehavior::Chase {
                        target: in_range[0].0,
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
                    approach(
                        transform,
                        velocity,
                        &mut force,
                        position,
                        thrust_power,
                        turn_speed,
                        100.0,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Wander => {}
            AIShipBehavior::Attack { target_position } => {
                approach(
                    transform,
                    velocity,
                    &mut force,
                    target_position,
                    thrust_power,
                    turn_speed,
                    100.0,
                );
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

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        op_position,
                        thrust_power,
                        turn_speed,
                        100.0,
                    );
                }
            }
            AIShipBehavior::Patrol {
                around_position,
                radius,
            } => {
                go_around(
                    transform,
                    velocity,
                    &mut force,
                    around_position,
                    radius,
                    thrust_power,
                    turn_speed,
                );
            }
            AIShipBehavior::Guard { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    go_around(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        radius,
                        thrust_power,
                        turn_speed,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Follow { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        thrust_power,
                        turn_speed,
                        radius,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Kamikaze { target } => {
                // Kamikaze at the target
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        thrust_power,
                        turn_speed,
                        0.0,
                    );
                } else {
                    controller.reset();
                }
            }
        }
    }
}

fn approach(
    transform: &Transform,
    velocity: &Velocity,
    force: &mut ExternalForce,
    target_position: Vec2,
    thrust_power: f32,
    turn_speed: f32,
    radius: f32,
) {
    let direction = (target_position - transform.translation.truncate()).normalize_or_zero();
    let angle = direction.y.atan2(direction.x) - PI / 2.0;
    let angle_diff = angle - transform.rotation.to_euler(EulerRot::XYZ).2;
    let distance = transform.translation.truncate().distance(target_position);

    let turn_speed_multiplier = (angle_diff.abs() / PI).clamp(0.25, 1.0);
    let force_multiplier = if distance > radius.squared().squared() {
        1.0
    } else if distance > radius.squared() {
        0.5
    } else {
        0.1
    };

    let torque = (if angle_diff > 0.0 {
        turn_speed
    } else {
        -turn_speed
    }) * (if velocity.linvel.length() > PI / 4.0 {
        0.1
    } else {
        1.0
    }) * turn_speed_multiplier;

    force.torque = torque;

    if angle_diff.abs() < PI / 2.0 {
        if distance <= radius {
            force.force = Vec2::ZERO;
        } else {
            force.force = direction * thrust_power * force_multiplier;
        }
    }
}

fn go_around(
    transform: &Transform,
    velocity: &Velocity,
    force: &mut ExternalForce,
    around_position: Vec2,
    radius: f32,
    thrust_power: f32,
    turn_speed: f32,
) {
    let in_range = transform.translation.truncate().distance(around_position) <= 300.0;

    let direction = (around_position - transform.translation.truncate()).normalize_or_zero();

    let position_on_ring = transform.translation.truncate() + direction * radius;

    if in_range {
        let next_position = position_on_ring + Vec2::new(direction.y, -direction.x) * 100.0;

        approach(
            transform,
            velocity,
            force,
            next_position,
            thrust_power,
            turn_speed,
            100.0,
        );
    } else {
        approach(
            transform,
            velocity,
            force,
            position_on_ring,
            thrust_power,
            turn_speed,
            100.0,
        );
    }
}
