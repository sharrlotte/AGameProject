use bevy::asset::AssetEvent;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};


use ron::ser::{PrettyConfig, to_string_pretty};

use std::env;

use std::fs;

use std::path::PathBuf;


#[derive(Asset, TypePath, Serialize, Deserialize, Resource, PartialEq, Clone, Debug)]
pub struct GameSetting {
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_bloom")]
    pub bloom: bool,
}


impl Default for GameSetting {
    fn default() -> Self {
        Self {
            volume: 0.5,
            bloom: true,
        }
    }
}


#[derive(Resource)]
pub struct SettingSaveTimer(pub Timer);


impl Default for SettingSaveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}


fn default_volume() -> f32 {
    0.5
}


fn default_bloom() -> bool {
    true
}


pub fn game_setting_bridge(
    mut events: MessageReader<AssetEvent<GameSetting>>,
    assets: Res<Assets<GameSetting>>,
    mut setting: ResMut<GameSetting>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(asset) = assets.get(id) {
            setting.set_if_neq(asset.clone());
        }
    }
}


pub fn save_setting_on_change(
    setting: Res<GameSetting>,
    time: Res<Time>,
    mut timer: ResMut<SettingSaveTimer>,
) {
    if setting.is_changed() {
        timer.0.reset();
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    let root = asset_root();
    let path = root.join("config/setting.ron");

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        error!(
            "Failed to create config directory {}: {:?}",
            parent.display(),
            e
        );

        return;
    }

    match to_string_pretty(&*setting, PrettyConfig::default()) {
        Ok(ron) => {
            if let Err(e) = fs::write(&path, ron) {
                error!("Failed to write {}: {:?}", path.display(), e);
            }
        }
        Err(e) => {
            error!("Failed to serialize settings: {:?}", e);
        }
    }
    timer.0.reset();
}


fn asset_root() -> PathBuf {
    env::var("BEVY_ASSET_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"))
}
