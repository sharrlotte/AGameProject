use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    SceneState,
    scene::game::{
        background::BackgroundPlugin,
        resource::GameVars,
        system::{
            handle_ship_button_click, setup_game, update_fps, update_ship_buttons,
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
                Update,
                (
                    update_fps,
                    update_turret_control_mode_button,
                    update_ship_buttons.run_if(on_timer(Duration::from_millis(100))),
                    handle_ship_button_click,
                )
                    .run_if(in_state(SceneState::Game)),
            )
            .add_plugins(BackgroundPlugin);
    }
}
