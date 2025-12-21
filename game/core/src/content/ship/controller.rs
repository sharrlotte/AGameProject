use std::f32::consts::PI;

use bevy::{math::FloatPow, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::content::{
    ship::base::{AIShipBehavior, AIShipController, PlayerShipController, Ship},
    team::Team,
};

const THRUST_POWER_FACTOR: f32 = 160.0;
const TURN_SPEED_FACTOR: f32 = 9000.0;

pub fn player_ship_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Transform, &Ship), With<PlayerShipController>>,
) {
    for (mut force, transform, ship) in query.iter_mut() {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        let mut linear_force = Vec2::ZERO;
        let mut torque = 0.0;

        let forward = transform.local_y().truncate(); // Assuming +Y is forward
        let right = transform.local_x().truncate();

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            linear_force += forward * thrust_power;
        }

        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            linear_force -= forward * thrust_power;
        }

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            torque += turn_speed;
        }

        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
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
}

pub fn ai_ship_controller(
    mut query: Query<(
        &mut AIShipController,
        &mut ExternalForce,
        &Velocity,
        &Transform,
        &Ship,
        &Team,
    )>,
    ships: Query<(Entity, &Transform, &Ship, &Team)>,
    global_transforms: Query<&Transform>,
) {
    for (mut controller, mut force, velocity, transform, ship, team) in query.iter_mut() {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        let enemy_ships: Vec<(Entity, &Transform, &Ship, &Team)> = ships
            .iter()
            .filter(|(_, _, _, other_team)| other_team.id != team.id)
            .collect();

        let in_range: Vec<(Entity, &Transform, &Ship, &Team)> = enemy_ships
            .into_iter()
            .filter(|(_, enemy_transform, _, _)| {
                enemy_transform
                    .translation
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
