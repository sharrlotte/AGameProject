use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use data::{GameState, PlayerConfig, SimUpdateSet};
use sim_world::{ActionIntent, LocalPlayer};

use crate::camera::MainCamera;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            gather_local_input
                .before(SimUpdateSet)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn gather_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    player_cfg: Res<PlayerConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut intents: Query<&mut ActionIntent, With<LocalPlayer>>,
) {
    let mut move_dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir.x += 1.0;
    }
    if move_dir.length_squared() > 1e-4 {
        move_dir = move_dir.normalize();
    }

    let sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let fire_held = mouse_buttons.pressed(MouseButton::Left);
    let fire_just_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    let reload_just_pressed = keys.just_pressed(KeyCode::KeyR);

    let swap_slot = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else {
        None
    };

    let aim_world = compute_cursor_aim_world(&windows, &camera_query, player_cfg.aim_height);

    for mut intent in &mut intents {
        intent.move_dir = move_dir;
        intent.sprint = sprint;
        if aim_world.is_some() {
            intent.aim_world = aim_world;
        }
        intent.fire_held = fire_held;
        if fire_just_pressed {
            intent.fire_just_pressed = true;
        }
        if reload_just_pressed {
            intent.reload_just_pressed = true;
        }
        if let Some(slot) = swap_slot {
            intent.swap_slot = Some(slot);
        }
    }
}

fn compute_cursor_aim_world(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    aim_height: f32,
) -> Option<Vec3> {
    let window = windows.iter().next()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, cam_transform) = camera_query.iter().next()?;
    let ray = camera.viewport_to_world(cam_transform, cursor_pos).ok()?;
    let plane = InfinitePlane3d::new(Vec3::Y);
    let distance = ray.intersect_plane(Vec3::new(0.0, aim_height, 0.0), plane)?;
    Some(ray.get_point(distance))
}
