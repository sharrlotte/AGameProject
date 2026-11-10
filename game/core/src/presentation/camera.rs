use bevy::prelude::*;

use crate::sim::input_intent::ActionIntent;
use crate::sim::player::LocalPlayer;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 22.0, 12.5);
const CAMERA_FOLLOW_SHARPNESS: f32 = 10.0;
const AIM_LEAD_FACTOR: f32 = 0.16;
const MAX_AIM_LEAD_METERS: f32 = 3.5;

#[derive(Component)]
pub struct MainCamera;

/// Spawns the angled top-down 3D camera.
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Smoothly tracks the local player with a subtle mouse aim-lead offset.
pub fn follow_local_player(
    time: Res<Time>,
    players: Query<(&Transform, &ActionIntent), With<LocalPlayer>>,
    mut cameras: Query<&mut Transform, (With<MainCamera>, Without<LocalPlayer>)>,
) {
    let Some((player_tf, intent)) = players.iter().next() else {
        return;
    };
    let Some(mut cam_tf) = cameras.iter_mut().next() else {
        return;
    };

    let mut aim_lead = Vec3::ZERO;
    if let Some(aim_world) = intent.aim_world {
        let delta = Vec3::new(
            aim_world.x - player_tf.translation.x,
            0.0,
            aim_world.z - player_tf.translation.z,
        );
        let scaled = delta * AIM_LEAD_FACTOR;
        let len = scaled.length();
        aim_lead = if len > MAX_AIM_LEAD_METERS {
            scaled * (MAX_AIM_LEAD_METERS / len)
        } else {
            scaled
        };
    }

    let focus = Vec3::new(player_tf.translation.x, 0.0, player_tf.translation.z) + aim_lead;
    let target_pos = focus + CAMERA_OFFSET;
    let alpha = 1.0 - (-CAMERA_FOLLOW_SHARPNESS * time.delta_secs()).exp();

    cam_tf.translation = cam_tf.translation.lerp(target_pos, alpha);
    cam_tf.look_at(focus, Vec3::Y);
}
