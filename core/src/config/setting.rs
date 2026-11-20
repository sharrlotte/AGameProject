use bevy::asset::AssetEvent;

use bevy::prelude::*;

use serde::{Deserialize, Serialize};


use ron::ser::{PrettyConfig, to_string_pretty};

use std::env;

use std::fs;

use std::path::PathBuf;

use crate::validate::{Validate, check};


#[derive(Asset, TypePath, Serialize, Deserialize, Resource, PartialEq, Clone, Debug)]
pub struct GameSetting {
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_bloom")]
    pub bloom: bool,
    #[serde(default = "default_ai_level")]
    pub ai_level: u8,
    #[serde(default)]
    pub keys: KeySettings,
}


#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct KeySettings {
    #[serde(default = "default_rotate_ccw")]
    pub rotate_ccw: KeyCode,
    #[serde(default = "default_rotate_cw")]
    pub rotate_cw: KeyCode,
}


impl Default for KeySettings {
    fn default() -> Self {
        Self {
            rotate_ccw: KeyCode::KeyQ,
            rotate_cw: KeyCode::KeyE,
        }
    }
}


impl Default for GameSetting {
    fn default() -> Self {
        Self {
            volume: 0.5,
            bloom: true,
            ai_level: default_ai_level(),
            keys: KeySettings::default(),
        }
    }
}

impl Validate for GameSetting {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::bounded_int("AI difficulty level", self.ai_level, 1, 10)
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


fn default_ai_level() -> u8 {
    10
}


fn default_rotate_ccw() -> KeyCode {
    KeyCode::KeyQ
}


fn default_rotate_cw() -> KeyCode {
    KeyCode::KeyE
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
            let mut next = asset.clone();
            next.ai_level = next.ai_level.clamp(1, 10);
            setting.set_if_neq(next);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_setting_parses_with_rotation_keys() {
        let setting: GameSetting = ron::from_str(include_str!("../../assets/config/setting.ron"))
            .expect("shipped setting parses");
        assert_eq!(setting.keys.rotate_ccw, KeyCode::KeyQ);
        assert_eq!(setting.keys.rotate_cw, KeyCode::KeyE);
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let setting: GameSetting = ron::from_str("(volume: 0.5, bloom: true)")
            .expect("setting without keys parses");
        assert_eq!(setting, GameSetting::default());
    }
}
