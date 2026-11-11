use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use world::{ActionIntent, LocalPlayer, PlayerConfig};

use crate::camera::{CameraConfig, CameraZoomMessage, MainCamera};

/// Gathers local keyboard/mouse inputs into [`ActionIntent`] and emits [`CameraZoomMessage`] on `Ctrl + Scroll`.
#[allow(clippy::too_many_arguments)]
pub fn gather_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut wheel_reader: MessageReader<MouseWheel>,
    player_cfg: Res<PlayerConfig>,
    cam_cfg: Res<CameraConfig>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut intents: Query<&mut ActionIntent, With<LocalPlayer>>,
    mut zoom_writer: MessageWriter<CameraZoomMessage>,
) {
    emit_camera_zoom(&keys, &mut wheel_reader, &cam_cfg, &mut zoom_writer);

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
    let aim_held = mouse_buttons.pressed(MouseButton::Right);
    let reload_just_pressed = keys.just_pressed(KeyCode::KeyR);

    let swap_slot = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else {
        None
    };

    let (cam, cam_transform) = *camera;
    let aim_world = compute_cursor_aim_world(&window, cam, cam_transform, player_cfg.aim_height);

    for mut intent in &mut intents {
        intent.move_dir = move_dir;
        intent.sprint = sprint;
        if aim_world.is_some() {
            intent.aim_world = aim_world;
        }
        intent.fire_held = fire_held;
        intent.aim_held = aim_held;
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

fn emit_camera_zoom(
    keys: &ButtonInput<KeyCode>,
    wheel_reader: &mut MessageReader<MouseWheel>,
    cam_cfg: &CameraConfig,
    zoom_writer: &mut MessageWriter<CameraZoomMessage>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut scroll_steps = 0.0;
    // Drain all wheel events every frame so un-modified scrolling does not queue up before Ctrl is held.
    for event in wheel_reader.read() {
        if !ctrl {
            continue;
        }
        scroll_steps += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / cam_cfg.zoom_pixel_divisor.max(1.0),
        };
    }
    if scroll_steps.abs() > 1e-4 {
        zoom_writer.write(CameraZoomMessage {
            steps: scroll_steps,
        });
    }
}

fn compute_cursor_aim_world(
    window: &Window,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    aim_height: f32,
) -> Option<Vec3> {
    let cursor_pos = window.cursor_position()?;
    let ray = camera.viewport_to_world(cam_transform, cursor_pos).ok()?;
    let plane = InfinitePlane3d::new(Vec3::Y);
    let distance = ray.intersect_plane(Vec3::new(0.0, aim_height, 0.0), plane)?;
    Some(ray.get_point(distance))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_input_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<PlayerConfig>()
            .init_resource::<CameraConfig>()
            .add_message::<MouseWheel>()
            .add_message::<CameraZoomMessage>()
            .add_systems(Update, gather_local_input);

        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app.world_mut()
            .spawn((MainCamera, Camera::default(), GlobalTransform::default()));
        app.world_mut()
            .spawn((LocalPlayer, ActionIntent::default()));
        app
    }

    #[test]
    fn scroll_without_ctrl_does_not_emit_zoom_message() {
        let mut app = setup_input_test_app();
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window: Entity::PLACEHOLDER,
        });
        app.update();

        let messages = app.world().resource::<Messages<CameraZoomMessage>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(cursor.read(messages).count(), 0);
    }

    #[test]
    fn ctrl_scroll_emits_zoom_message() {
        let mut app = setup_input_test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.5,
            window: Entity::PLACEHOLDER,
        });
        app.update();

        let messages = app.world().resource::<Messages<CameraZoomMessage>>();
        let mut cursor = messages.get_cursor();
        let emitted: Vec<_> = cursor.read(messages).collect();
        assert_eq!(emitted.len(), 1);
        assert!((emitted[0].steps - 1.5).abs() < 1e-5);
    }
}
