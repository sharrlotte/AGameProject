use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::{
    ships::{AIShipBehavior, AIShipController, PlayerShipController},
    structures::ship::Ship,
    team::Team,
};

const THRUST_POWER_FACTOR: f32 = 160.0;
const TURN_SPEED_FACTOR: f32 = 12000.0;

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
        &mut ExternalForce,
        &Transform,
        &mut AIShipController,
        &Ship,
        &Team,
    )>,
    ships: Query<(Entity, &Transform, &Ship, &Team)>,
) {
    for (mut force, transform, mut controller, ship, team) in query.iter_mut() {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        let enemy_ships: Vec<(Entity, &Transform, &Ship, &Team)> = ships
            .iter()
            .filter(|(_, _, _, other_team)| other_team.id != team.id)
            .collect();

        let in_range = enemy_ships.iter().any(|(_, enemy_transform, _, _)| {
            enemy_transform
                .translation
                .truncate()
                .distance(transform.translation.truncate())
                <= ship.radar_range
        });

        match controller.behavior {
            AIShipBehavior::Idle => {
                if in_range {
                    controller.behavior = AIShipBehavior::Chase {
                        target: enemy_ships[0].0,
                    };
                }
            }
            AIShipBehavior::Chase { target } => {
                let position = enemy_ships
                    .iter()
                    .find(|(entity, _, _, _)| *entity == target)
                    .map(|(_, transform, _, _)| transform.translation.truncate());

                if let Some(position) = position {
                    move_to(transform, &mut force, position, thrust_power, turn_speed);
                } else {
                    controller.behavior = controller.initial_behavior;
                }
            }
            AIShipBehavior::Wander => {
                // Randomly move around
            }
            AIShipBehavior::Attack { target_pos } => {
                move_to(transform, &mut force, target_pos, thrust_power, turn_speed);
            }
            AIShipBehavior::Flee => {
                // Run away from the player
            }
            AIShipBehavior::Patrol => {
                // Move between predefined points
            }
        }
    }
}

const CLOSE_ENOUGH_DISTANCE: f32 = 300.0;

fn move_to(
    transform: &Transform,
    force: &mut ExternalForce,
    target_pos: Vec2,
    thrust_power: f32,
    turn_speed: f32,
) {
    if transform.translation.truncate().distance(target_pos) <= CLOSE_ENOUGH_DISTANCE {
        force.force = Vec2::ZERO;
        force.torque = 0.0;
        return;
    }

    let direction = (target_pos - transform.translation.truncate()).normalize_or_zero();
    let angle = direction.y.atan2(direction.x) - PI / 2.0;
    let angle_diff = angle - transform.rotation.to_euler(EulerRot::XYZ).2;

    let percent = (angle_diff.abs() / (PI / 2.0)).clamp(0.25, 1.0);

    let torque = if angle_diff > 0.0 {
        turn_speed * percent
    } else {
        -turn_speed * percent
    };

    force.torque = torque;

    if angle_diff.abs() < PI / 2.0 {
        let linear_force = direction * thrust_power;
        force.force = linear_force;
    }
}
