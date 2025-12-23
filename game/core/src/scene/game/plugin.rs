use bevy::prelude::*;

use crate::{
    SceneState,
    scene::game::{
        background::BackgroundPlugin,
        resource::GameVars,
        system::{
            cleanup_game, cleanup_game_entities, setup_game, update_fps,
            update_turret_control_mode_button,
        },
    },
};

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameVars>()
            .add_systems(OnEnter(SceneState::Game), setup_game)
            .add_systems(
                OnExit(SceneState::Game),
                (cleanup_game, cleanup_game_entities),
            )
            .add_systems(
                Update,
                (update_fps, update_turret_control_mode_button).run_if(in_state(SceneState::Game)),
            )
            .add_plugins(BackgroundPlugin);
    }
}
