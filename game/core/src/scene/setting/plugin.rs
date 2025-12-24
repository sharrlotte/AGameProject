use bevy::prelude::*;

use crate::scene::setting::resource::GameSetting;

pub struct SettingScenePlugin;

impl Plugin for SettingScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSetting>();
    }
}
