use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::save::document::{data_dir, read_document, write_document};
use crate::validate::{Validate, check};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Resource, PartialEq, Clone, Debug)]
pub struct GameSettings {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_bloom")]
    pub bloom: bool,
    #[serde(default = "default_ai_level")]
    pub ai_level: u8,
    #[serde(default)]
    pub keys: KeySettings,
}

crate::ron_document!(GameSettings, SETTINGS_SCHEMA_VERSION, "game settings");

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

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_SCHEMA_VERSION,
            volume: default_volume(),
            bloom: default_bloom(),
            ai_level: default_ai_level(),
            keys: KeySettings::default(),
        }
    }
}

impl Validate for GameSettings {
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

fn default_version() -> u32 {
    SETTINGS_SCHEMA_VERSION
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

pub fn settings_path_in(base: &Path) -> PathBuf {
    base.join("settings.ron")
}

pub fn settings_path() -> PathBuf {
    settings_path_in(&data_dir())
}

pub fn load_settings_in(base: &Path) -> GameSettings {
    let path = settings_path_in(base);
    if !path.exists() {
        return GameSettings::default();
    }
    match read_document::<GameSettings>(&path) {
        Ok(mut loaded) => {
            loaded.ai_level = loaded.ai_level.clamp(1, 10);
            loaded
        }
        Err(e) => {
            warn!("Failed to read {}: {e}; using defaults", path.display());
            GameSettings::default()
        }
    }
}

pub fn load_settings(mut commands: Commands) {
    commands.insert_resource(load_settings_in(&data_dir()));
}

pub fn save_setting_on_change(
    settings: Res<GameSettings>,
    time: Res<Time>,
    mut timer: ResMut<SettingSaveTimer>,
) {
    if settings.is_changed() {
        timer.0.reset();
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.is_finished() {
        return;
    }

    let path = settings_path();
    if let Err(e) = write_document(&path, &*settings) {
        error!("Failed to write {}: {e}", path.display());
    }
    timer.0.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("agame-settings-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn missing_settings_falls_back_to_defaults() {
        let dir = temp_dir("missing");
        assert_eq!(load_settings_in(&dir), GameSettings::default());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn settings_round_trip_atomically_in_data_dir() {
        let dir = temp_dir("roundtrip");
        let path = settings_path_in(&dir);
        let mut settings = GameSettings::default();
        settings.volume = 0.25;
        settings.bloom = false;
        settings.ai_level = 7;
        write_document(&path, &settings).expect("write settings");

        let loaded = load_settings_in(&dir);
        assert_eq!(loaded, settings);
        let _ = fs::remove_dir_all(dir);
    }
}
