use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::GameConfigValue;
use crate::content::ship::{Ship, Throttle};
use crate::content::ship::stat::ShipStats;
use crate::content::team::Team;
use crate::control::{CameraTarget, ControlMode, ControlledShip};

pub fn player_control_ship(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut ships: Query<
        (&mut ExternalForce, &Transform, &Ship, &ShipStats, &mut Throttle),
        Without<Camera2d>,
    >,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_target: ResMut<CameraTarget>,
    mut controlled: ResMut<ControlledShip>,
    mode: Res<ControlMode>,
    config: Res<GameConfigValue>,
) {
    for event in mouse_wheel_events.read() {
        if let Ok((_, mut projection)) = camera.single_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(config.0.min_zoom, config.0.max_zoom);
            }
        }
    }

    let panning = mouse_button_input.pressed(MouseButton::Middle);
    let drag_delta = if panning {
        mouse_motion
            .read()
            .fold(Vec2::ZERO, |acc, event| acc + event.delta)
    } else {
        Vec2::ZERO
    };

    if panning {
        if let Ok((mut camera_transform, projection)) = camera.single_mut() {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };
            camera_transform.translation.x -= drag_delta.x * scale;
            camera_transform.translation.y -= drag_delta.y * scale;
        }
    } else if let Some(following) = camera_target.following {
        match ships.get(following) {
            Ok((_, transform, ship, _, _)) => {
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
                camera_target.following = None;
            }
        }
    } else if *mode == ControlMode::FreeLook {
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

    if *mode == ControlMode::Pilot {
        if let Some(controlling) = controlled.controlling {
            match ships.get_mut(controlling) {
                Ok((mut force, transform, _, stats, mut throttle)) => {
                    let thrust_power = stats.thrust_power;
                    let turn_speed = stats.rotate_speed;

                    let mut linear_force = Vec2::ZERO;
                    let mut torque = 0.0;
                    let mut linear_active = false;
                    let mut angular_active = false;

                    let forward = transform.local_y().truncate();
                    let right = transform.local_x().truncate();

                    if keyboard_input.pressed(KeyCode::KeyW)
                        || keyboard_input.pressed(KeyCode::ArrowUp)
                    {
                        linear_force += forward * thrust_power;
                        linear_active = true;
                    }

                    if keyboard_input.pressed(KeyCode::KeyS)
                        || keyboard_input.pressed(KeyCode::ArrowDown)
                    {
                        linear_force -= forward * thrust_power;
                        linear_active = true;
                    }

                    if keyboard_input.pressed(KeyCode::KeyA)
                        || keyboard_input.pressed(KeyCode::ArrowLeft)
                    {
                        torque += turn_speed;
                        angular_active = true;
                    }

                    if keyboard_input.pressed(KeyCode::KeyD)
                        || keyboard_input.pressed(KeyCode::ArrowRight)
                    {
                        torque -= turn_speed;
                        angular_active = true;
                    }

                    if keyboard_input.pressed(KeyCode::KeyQ) {
                        linear_force -= right * thrust_power;
                        linear_active = true;
                    }
                    if keyboard_input.pressed(KeyCode::KeyE) {
                        linear_force += right * thrust_power;
                        linear_active = true;
                    }

                    force.force = linear_force;
                    force.torque = torque;
                    throttle.set(
                        if linear_active { 1.0 } else { 0.0 },
                        if angular_active { 1.0 } else { 0.0 },
                    );
                }
                Err(_) => {
                    controlled.controlling = None;
                }
            }
        }
    } else if let Some(controlling) = controlled.controlling {
        if let Ok((mut force, _, _, _, mut throttle)) = ships.get_mut(controlling) {
            force.force = Vec2::ZERO;
            force.torque = 0.0;
            throttle.set(0.0, 0.0);
        }
    }
}

pub fn toggle_control_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<ControlMode>,
    mut camera_target: ResMut<CameraTarget>,
    mut controlled: ResMut<ControlledShip>,
    ships: Query<(Entity, &Team), With<Ship>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    match *mode {
        ControlMode::Pilot => {
            *mode = ControlMode::FreeLook;
            controlled.controlling = None;
            camera_target.following = None;
        }
        ControlMode::FreeLook => {
            let target = camera_target.following.or_else(|| {
                ships
                    .iter()
                    .find(|(_, team)| team.is_player())
                    .map(|(entity, _)| entity)
            });
            if let Some(target) = target {
                controlled.controlling = Some(target);
                camera_target.following = Some(target);
                *mode = ControlMode::Pilot;
            }
        }
    }
}
