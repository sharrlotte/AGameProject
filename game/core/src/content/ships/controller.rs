use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::ships::{AIShipController, PlayerShipController};

pub fn player_ship_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Transform), With<PlayerShipController>>,
) {
    let thrust_power = 160_000.0; // Needs tuning based on mass
    let turn_speed = 3_000_000.0; // Needs tuning

    for (mut ext_force, transform) in query.iter_mut() {
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

        ext_force.force = linear_force;
        ext_force.torque = torque;
    }
}

pub fn ai_ship_controller(
    mut query: Query<(&mut ExternalForce, &Transform), With<AIShipController>>,
) {
    // TODO: Implement AI ship controller
}
