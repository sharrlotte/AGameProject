use bevy::prelude::*;
#[cfg(feature = "dev")]
use data::{GameState, GameScene};

pub mod hud;
pub mod menu;
pub mod theme;

pub use hud::HudPlugin;
pub use menu::MenusPlugin;

#[cfg(feature = "dev")]
#[derive(Message, Clone, Copy)]
pub(super) struct ToggleAnimTestSceneMessage;

/// Registers all HUD and menu UI plugins.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((HudPlugin, MenusPlugin));

        #[cfg(feature = "dev")]
        app.add_message::<ToggleAnimTestSceneMessage>().add_systems(
            Update,
            (handle_anim_test_shortcut, handle_toggle_anim_test_scene)
                .chain()
                .run_if(not(in_state(GameState::Loading))),
        );
    }
}

#[cfg(feature = "dev")]
fn handle_anim_test_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<ToggleAnimTestSceneMessage>,
) {
    if keys.just_pressed(KeyCode::F8) {
        writer.write(ToggleAnimTestSceneMessage);
    }
}

#[cfg(feature = "dev")]
fn handle_toggle_anim_test_scene(
    mut reader: MessageReader<ToggleAnimTestSceneMessage>,
    game_state: Res<State<GameState>>,
    scene: Res<State<GameScene>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_scene: ResMut<NextState<GameScene>>,
) {
    if reader.read().next().is_none() {
        return;
    }

    if *game_state.get() == GameState::Playing {
        let target_scene = if *scene.get() == GameScene::AnimTest {
            GameScene::Arena
        } else {
            GameScene::AnimTest
        };
        next_scene.set(target_scene);
    } else {
        next_scene.set(GameScene::AnimTest);
        next_game_state.set(GameState::Playing);
    }
}
