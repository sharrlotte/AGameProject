use bevy::prelude::*;

use crate::{
    SceneState,
    scene::game::{
        background::BackgroundPlugin,
        system::{
            camera_follow, camera_zoom, cleanup_game, cleanup_game_entities, setup_game,
            update_fps, update_turret_control_mode_button,
        },
    },
};

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Game), setup_game)
            .add_systems(
                OnExit(SceneState::Game),
                (cleanup_game, cleanup_game_entities),
            )
            .add_systems(
                Update,
                (
                    camera_follow,
                    camera_zoom,
                    update_fps,
                    update_turret_control_mode_button,
                )
                    .run_if(in_state(SceneState::Game)),
            )
            .add_plugins(BackgroundPlugin);
    }
}
