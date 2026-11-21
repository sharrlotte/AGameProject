use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::save::document::{data_dir, read_document, write_document};
use crate::validate::SkillLevel;

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Resource, PartialEq, Clone, Debug)]
#[serde(default)]
pub struct GameSettings {
    pub version: u32,
    pub volume: f32,
    pub bloom: bool,
    #[serde(deserialize_with = "deserialize_skill_saturating")]
    pub ai_level: SkillLevel,
    pub keys: KeySettings,
}

fn deserialize_skill_saturating<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<SkillLevel, D::Error> {
    let raw = u8::deserialize(deserializer)?;
    match SkillLevel::new(raw) {
        Ok(level) => Ok(level),
        Err(_) => {
            warn!("Saved ai_level {raw} is out of 1..=10; clamping");
            Ok(SkillLevel::saturating(raw as i16))
        }
    }
}

crate::ron_document!(GameSettings, SETTINGS_SCHEMA_VERSION, "game settings");

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(default)]
pub struct KeySettings {
    pub rotate_ccw: KeyCode,
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
            volume: 0.5,
            bloom: true,
            ai_level: SkillLevel::MAX,
            keys: KeySettings::default(),
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
        Ok(loaded) => loaded,
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
        settings.ai_level = SkillLevel::new(7).expect("7 is valid");
        write_document(&path, &settings).expect("write settings");

        let loaded = load_settings_in(&dir);
        assert_eq!(loaded, settings);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn out_of_range_ai_level_clamps_without_resetting_other_fields() {
        let dir = temp_dir("clamp");
        let path = settings_path_in(&dir);
        fs::write(&path, "(version: 1, volume: 0.2, bloom: false, ai_level: 99)")
            .expect("write raw settings");

        let loaded = load_settings_in(&dir);
        assert!((loaded.volume - 0.2).abs() < 1e-6);
        assert!(!loaded.bloom);
        assert_eq!(loaded.ai_level, SkillLevel::MAX);
        let _ = fs::remove_dir_all(dir);
    }
}
