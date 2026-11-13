use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::world::{ActionIntent, LocalPlayer, PlayerConfig};

use crate::presentation::camera::{CameraConfig, CameraZoomMessage, MainCamera};

/// Gathers local keyboard/mouse inputs into [`ActionIntent`] and emits [`CameraZoomMessage`] on `Ctrl + Scroll`.
#[allow(clippy::too_many_arguments)]
pub fn gather_local_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut wheel_reader: MessageReader<MouseWheel>,
    player_cfg: Res<PlayerConfig>,
    cam_cfg: Res<CameraConfig>,
    window: Single<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut intents: Query<&mut ActionIntent, With<LocalPlayer>>,
    mut zoom_writer: MessageWriter<CameraZoomMessage>,
) {
    let mut raw = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        raw.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        raw.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        raw.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        raw.x += 1.0;
    }
    if raw.length_squared() > 1e-4 {
        raw = raw.normalize();
    }

    let camera = cameras.single().ok();
    let forward = camera
        .map(|(_, transform)| transform.forward().as_vec3())
        .unwrap_or(Vec3::NEG_Z);
    let world = move_to_world(raw, forward);
    let move_dir = Vec2::new(world.x, world.z);

    let sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let fire_held = mouse_buttons.pressed(MouseButton::Left);
    let fire_just_pressed = mouse_buttons.just_pressed(MouseButton::Left);
    let aim_held = mouse_buttons.pressed(MouseButton::Right);
    let reload_just_pressed = keys.just_pressed(KeyCode::KeyR);

    let weapon_scroll = emit_camera_zoom(&keys, &mut wheel_reader, &cam_cfg, &mut zoom_writer);

    let swap_slot = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else {
        None
    };
    let swap_next = weapon_scroll > 0.0;
    let swap_prev = weapon_scroll < 0.0;

    let aim_world = camera.and_then(|(cam, transform)| {
        compute_cursor_aim_world(&window, cam, transform, player_cfg.aim_height)
    });

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
        intent.swap_next = swap_next;
        intent.swap_prev = swap_prev;
    }
}

fn emit_camera_zoom(
    keys: &ButtonInput<KeyCode>,
    wheel_reader: &mut MessageReader<MouseWheel>,
    cam_cfg: &CameraConfig,
    zoom_writer: &mut MessageWriter<CameraZoomMessage>,
) -> f32 {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut scroll_steps = 0.0;
    let mut weapon_scroll = 0.0;
    // Drain all wheel events every frame so un-modified scrolling does not queue up before Ctrl is held.
    for event in wheel_reader.read() {
        let step = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / cam_cfg.zoom_pixel_divisor.max(1.0),
        };
        if ctrl {
            scroll_steps += step;
        } else {
            weapon_scroll += step;
        }
    }
    if scroll_steps.abs() > 1e-4 {
        zoom_writer.write(CameraZoomMessage {
            steps: scroll_steps,
        });
    }
    weapon_scroll
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

/// Projects input onto the camera yaw basis, falling back to world axes.
fn move_to_world(input: Vec2, forward: Vec3) -> Vec3 {
    let flat = Vec3::new(forward.x, 0.0, forward.z);
    let forward = if flat.length_squared() > 1e-6 {
        flat.normalize()
    } else {
        Vec3::NEG_Z
    };
    let right = forward.cross(Vec3::Y);
    let right = if right.length_squared() > 1e-6 {
        right.normalize()
    } else {
        Vec3::X
    };
    forward * input.y + right * input.x
}

#[cfg(test)]
mod tests {
    use bevy::input::touch::TouchPhase;

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
            phase: TouchPhase::Moved,
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
            phase: TouchPhase::Moved,
        });
        app.update();

        let messages = app.world().resource::<Messages<CameraZoomMessage>>();
        let mut cursor = messages.get_cursor();
        let emitted: Vec<_> = cursor.read(messages).collect();
        assert_eq!(emitted.len(), 1);
        assert!((emitted[0].steps - 1.5).abs() < 1e-5);
    }

    #[test]
    fn movement_is_projected_onto_the_camera_basis() {
        let forward = move_to_world(Vec2::new(0.0, 1.0), Vec3::X);
        assert!((forward - Vec3::X).length() < 1e-4);

        let right = move_to_world(Vec2::new(1.0, 0.0), Vec3::X);
        assert!((right - Vec3::Z).length() < 1e-4);
    }

    #[test]
    fn movement_falls_back_to_world_axes_without_a_camera() {
        let forward = move_to_world(Vec2::new(0.0, 1.0), Vec3::ZERO);
        assert!((forward - Vec3::NEG_Z).length() < 1e-4);

        let right = move_to_world(Vec2::new(1.0, 0.0), Vec3::Y);
        assert!((right - Vec3::X).length() < 1e-4);
    }

    #[test]
    fn w_moves_along_the_camera_forward() {
        let mut app = setup_input_test_app();
        {
            let mut cameras = app
                .world_mut()
                .query_filtered::<&mut GlobalTransform, With<MainCamera>>();
            let mut transform = cameras
                .single_mut(app.world_mut())
                .expect("main camera transform");
            *transform =
                GlobalTransform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2));
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();

        let mut intents = app
            .world_mut()
            .query_filtered::<&ActionIntent, With<LocalPlayer>>();
        let intent = intents.single(app.world()).expect("local player intent");
        assert!(
            (intent.move_dir - Vec2::new(1.0, 0.0)).length() < 1e-4,
            "expected +X, got {:?}",
            intent.move_dir
        );
    }
}







