use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use data::{ArenaConfig, ConfigHotReloadedMessage, GameState};
use sim_world::{ActionIntent, LocalPlayer};

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), spawn_camera)
            .add_systems(
                Update,
                sync_camera_on_hot_reload.run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                PostUpdate,
                follow_local_player
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_camera(mut commands: Commands, cfg: Res<ArenaConfig>) {
    commands.spawn((
        Name::new("MainCamera"),
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(cfg.camera_offset).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn sync_camera_on_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<ArenaConfig>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }
    for mut tf in &mut cameras {
        let focus = tf.translation - cfg.camera_offset;
        *tf = Transform::from_translation(focus + cfg.camera_offset).looking_at(focus, Vec3::Y);
    }
}

fn follow_local_player(
    time: Res<Time>,
    cfg: Res<ArenaConfig>,
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
        let scaled = delta * cfg.camera_aim_lead_factor;
        let len = scaled.length();
        aim_lead = if len > cfg.camera_max_aim_lead {
            scaled * (cfg.camera_max_aim_lead / len)
        } else {
            scaled
        };
    }

    let focus = Vec3::new(player_tf.translation.x, 0.0, player_tf.translation.z) + aim_lead;
    let target_pos = focus + cfg.camera_offset;
    let alpha = 1.0 - (-cfg.camera_follow_sharpness * time.delta_secs()).exp();

    // Keep camera orientation fixed; only translate so follow lag never tilts/jiggles the view.
    cam_tf.translation = cam_tf.translation.lerp(target_pos, alpha);
}
