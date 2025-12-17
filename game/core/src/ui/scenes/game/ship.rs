use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::ships::spawner;
use crate::content::structures::registry::BlockRegistry;
use crate::content::structures::ship::PlayerShip;
use crate::content::team::Team;
use crate::{AppState, AppVar};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), (spawn_player_ship, spawn_enemy_ship))
            .add_systems(
                Update,
                ship_movement_controls.run_if(in_state(AppState::Game)),
            );
    }
}

pub fn spawn_player_ship(
    mut commands: Commands,
    app_var: Res<AppVar>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
) {
    let ship_data = if let Some(ship) = &app_var.current_ship {
        ship
    } else {
        warn!("No ship loaded in AppVar!");
        return;
    };

    spawner::spawn_ship(
        &mut commands,
        &registry,
        &asset_server,
        ship_data,
        Transform::from_translation(Vec3::ZERO),
        Team::new(0), // Ally
        true,         // is_player
    );
}

pub fn spawn_enemy_ship(
    mut commands: Commands,
    app_var: Res<AppVar>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
) {
    let ship_data = if let Some(ship) = &app_var.current_ship {
        ship
    } else {
        return;
    };

    // Spawn an enemy at (500, 500)
    spawner::spawn_ship(
        &mut commands,
        &registry,
        &asset_server,
        ship_data,
        Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        Team::new(1), // Enemy
        false,        // is_player
    );
}

pub fn ship_movement_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Transform), With<PlayerShip>>,
) {
    let thrust_power = 80_000.0; // Needs tuning based on mass
    let turn_speed = 1600_000.0; // Needs tuning

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
