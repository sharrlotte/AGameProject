use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::config::{ConfigError, ConfigFile, ConfigState};
use crate::validate::{Validate, check};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MinimapConfig {
    pub size_px: f32,
    pub margin_px: f32,
    pub fallback_world_radius: f32,
    pub radar_scale_factor: f32,
    pub refresh_secs: f32,
    pub dot_size_px: f32,
    pub lost_flash_count: u32,
    pub lost_flash_secs: f32,
    pub glow_blur_px: f32,
    pub max_markers: usize,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            size_px: 200.0,
            margin_px: 12.0,
            fallback_world_radius: 6000.0,
            radar_scale_factor: 1.2,
            refresh_secs: 1.0,
            dot_size_px: 4.0,
            lost_flash_count: 3,
            lost_flash_secs: 0.5,
            glow_blur_px: 2.0,
            max_markers: 128,
        }
    }
}

impl Validate for MinimapConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::positive("minimap size", self.size_px)?;
        check::positive("fallback world radius", self.fallback_world_radius)?;
        check::positive("radar scale factor", self.radar_scale_factor)?;
        check::positive("refresh interval", self.refresh_secs)?;
        check::positive("marker dot size", self.dot_size_px)?;
        check::positive("lost contact flash duration", self.lost_flash_secs)?;
        check::positive("glow blur radius", self.glow_blur_px)?;
        check::non_negative("screen margin", self.margin_px)?;
        check::at_least("lost contact flash count", self.lost_flash_count, 1)?;
        check::at_least("maximum marker count", self.max_markers, 1)?;
        Ok(())
    }
}

impl MinimapConfig {
    pub fn validate(&self) -> Result<(), String> {
        Validate::validate(self, &())
    }

    pub fn radius_px(&self) -> f32 {
        self.size_px * 0.5
    }

    pub fn scaled_world_radius(&self, radar_range: Option<f32>) -> f32 {
        match radar_range {
            Some(range) if range.is_finite() && range > 0.0 => range * self.radar_scale_factor,
            _ => self.fallback_world_radius,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct MinimapConfigValue(pub MinimapConfig);

pub fn minimap_config_bridge(
    mut events: MessageReader<AssetEvent<MinimapConfig>>,
    assets: Res<Assets<MinimapConfig>>,
    mut value: ResMut<MinimapConfigValue>,
    mut config_state: ResMut<NextState<ConfigState>>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
            if let Err(reason) = config.validate() {
                config_state.set(ConfigState::Error(ConfigError::with_detail(
                    ConfigFile::Minimap,
                    reason,
                )));
                continue;
            }
            *value = MinimapConfigValue(config.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_minimap_config_parses_and_validates() {
        let text = std::fs::read_to_string("assets/config/base/minimap.ron")
            .or_else(|_| std::fs::read_to_string("core/assets/config/base/minimap.ron"))
            .expect("minimap.ron should be readable from either repo or core root");
        let config: MinimapConfig =
            ron::from_str(&text).expect("minimap.ron should parse as MinimapConfig");
        config.validate().expect("shipped minimap.ron must pass validation");
    }

    #[test]
    fn non_positive_size_fails_validation() {
        let mut config = MinimapConfig::default();
        config.size_px = 0.0;
        let err = config.validate().expect_err("size_px 0 must fail");
        assert!(err.contains("minimap size"));
    }

    #[test]
    fn zero_lost_flash_count_fails_validation() {
        let mut config = MinimapConfig::default();
        config.lost_flash_count = 0;
        let err = config.validate().expect_err("lost_flash_count 0 must fail");
        assert!(err.contains("lost contact flash count"));
    }
}
