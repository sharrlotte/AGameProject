use bevy::prelude::*;

use crate::resource::SettingState;
use crate::scene::setting::resource::GameSetting;
use crate::scene::setting::system::*;

pub struct SettingScenePlugin;

impl Plugin for SettingScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingState>()
            .init_resource::<GameSetting>()
            .add_systems(OnEnter(SettingState::Open), setup_setting_ui)
            .add_systems(OnExit(SettingState::Open), cleanup_setting_ui)
            .add_systems(
                Update,
                (
                    handle_close_button,
                    handle_volume_change,
                    handle_bloom_toggle,
                    sync_settings_to_ui,
                )
                    .run_if(in_state(SettingState::Open)),
            )
            .add_systems(Update, apply_settings);
    }
}
