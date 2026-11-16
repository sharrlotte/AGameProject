use bevy::asset::AssetEvent;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

#[derive(Asset, TypePath, Serialize, Deserialize, Resource, PartialEq, Clone, Debug)]
pub struct GameSetting {
    pub volume: f32,
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

pub fn save_setting_on_change(setting: Res<GameSetting>) {
    if !setting.is_changed() {
        return;
    }

    let root = asset_root();
    let path = root.join("config/settings.ron");

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        error!("Failed to create config directory {}: {:?}", parent.display(), e);

        return;
    }

    match ron::ser::to_string_pretty(&*setting, ron::ser::PrettyConfig::default()) {
        Ok(ron) => {
            if let Err(e) = std::fs::write(&path, ron) {
                error!("Failed to write {}: {:?}", path.display(), e);
            }
        }
        Err(e) => {
            error!("Failed to serialize settings: {:?}", e);
        }
    }
}

fn asset_root() -> PathBuf {
    std::env::var("BEVY_ASSET_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"))
}
