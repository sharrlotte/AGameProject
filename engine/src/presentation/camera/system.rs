use bevy::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::world::{ActionIntent, LocalPlayer};

use crate::config::CameraConfig;
use crate::message::messages::CameraZoomMessage;
use super::state::{CameraZoom, MainCamera};

/// Spawns the primary 3D camera using the configured offset and default zoom.
pub fn spawn_camera(mut commands: Commands, cfg: Res<CameraConfig>) {
    let zoom = CameraZoom::from_config(&cfg);
    let offset = cfg.offset * zoom.current;
    commands.spawn((
        Name::new("MainCamera"),
        MainCamera,
        zoom,
        Camera3d::default(),
        Transform::from_translation(offset).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Applies incoming [`CameraZoomMessage`] steps to the main camera's target zoom.
pub fn apply_camera_zoom(
    mut zoom_reader: MessageReader<CameraZoomMessage>,
    cfg: Res<CameraConfig>,
    mut zoom: Single<&mut CameraZoom, With<MainCamera>>,
) {
    for msg in zoom_reader.read() {
        zoom.adjust_target(msg.steps, &cfg);
    }
}

/// Re-orients the camera and clamps zoom bounds when [`CameraConfig`] hot-reloads.
pub fn sync_camera_on_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<CameraConfig>,
    camera: Single<(&mut Transform, &mut CameraZoom), With<MainCamera>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }
    let (mut tf, mut zoom) = camera.into_inner();
    zoom.clamp_to_config(&cfg);
    let scaled_offset = cfg.offset * zoom.current;
    let focus = tf.translation - scaled_offset;
    *tf = Transform::from_translation(focus + scaled_offset).looking_at(focus, Vec3::Y);
}

/// Smoothly follows the local player with aim-lead and exponential zoom damping.
pub fn follow_local_player(
    time: Res<Time>,
    cfg: Res<CameraConfig>,
    player: Single<(&Transform, &ActionIntent), With<LocalPlayer>>,
    camera: Single<(&mut Transform, &mut CameraZoom), (With<MainCamera>, Without<LocalPlayer>)>,
) {
    let (player_tf, intent) = *player;
    let (mut cam_tf, mut zoom) = camera.into_inner();

    let mut aim_lead = Vec3::ZERO;
    if let Some(aim_world) = intent.aim_world {
        let delta = Vec3::new(
            aim_world.x - player_tf.translation.x,
            0.0,
            aim_world.z - player_tf.translation.z,
        );
        let scaled = delta * cfg.aim_lead_factor;
        let len = scaled.length();
        aim_lead = if len > cfg.max_aim_lead {
            scaled * (cfg.max_aim_lead / len)
        } else {
            scaled
        };
    }

    let dt = time.delta_secs();
    let zoom_alpha = 1.0 - (-cfg.zoom_sharpness * dt).exp();
    zoom.current += (zoom.target - zoom.current) * zoom_alpha;

    let focus = Vec3::new(player_tf.translation.x, 0.0, player_tf.translation.z) + aim_lead;
    let target_pos = focus + cfg.offset * zoom.current;
    let follow_alpha = 1.0 - (-cfg.follow_sharpness * dt).exp();

    // Keep camera orientation fixed; only translate so follow lag never tilts/jiggles the view.
    cam_tf.translation = cam_tf.translation.lerp(target_pos, follow_alpha);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_adjust_respects_direction_and_clamps_bounds() {
        let cfg = CameraConfig::default();
        let mut zoom = CameraZoom::from_config(&cfg);

        // Positive steps zoom in (decrease multiplier).
        zoom.adjust_target(1.0, &cfg);
        assert!((zoom.target - (cfg.zoom_default - cfg.zoom_step)).abs() < 1e-5);

        // Massive zoom-in clamps to zoom_min.
        zoom.adjust_target(100.0, &cfg);
        assert!((zoom.target - cfg.zoom_min).abs() < 1e-5);

        // Massive zoom-out clamps to zoom_max.
        zoom.adjust_target(-100.0, &cfg);
        assert!((zoom.target - cfg.zoom_max).abs() < 1e-5);
    }

    #[test]
    fn apply_camera_zoom_message_updates_target_zoom() {
        let mut app = App::new();
        app.add_message::<CameraZoomMessage>()
            .init_resource::<CameraConfig>()
            .add_systems(Update, apply_camera_zoom);

        let cfg = app.world().resource::<CameraConfig>().clone();
        let cam = app
            .world_mut()
            .spawn((MainCamera, CameraZoom::from_config(&cfg)))
            .id();

        app.world_mut().write_message(CameraZoomMessage { steps: 2.0 });
        app.update();

        let zoom = app.world().get::<CameraZoom>(cam).expect("camera zoom");
        assert!((zoom.target - (cfg.zoom_default - 2.0 * cfg.zoom_step)).abs() < 1e-5);
    }
}

