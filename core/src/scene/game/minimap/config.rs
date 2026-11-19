use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::config::{ConfigError, ConfigFile, ConfigState};

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
            dot_size_px: 6.0,
            lost_flash_count: 3,
            lost_flash_secs: 1.5,
            glow_blur_px: 4.0,
            max_markers: 128,
        }
    }
}

impl MinimapConfig {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive("size_px", self.size_px)?;
        validate_positive("fallback_world_radius", self.fallback_world_radius)?;
        validate_positive("radar_scale_factor", self.radar_scale_factor)?;
        validate_positive("refresh_secs", self.refresh_secs)?;
        validate_positive("dot_size_px", self.dot_size_px)?;
        validate_positive("lost_flash_secs", self.lost_flash_secs)?;
        validate_positive("glow_blur_px", self.glow_blur_px)?;

        if !self.margin_px.is_finite() || self.margin_px < 0.0 {
            return Err(format!(
                "margin_px must be non-negative, got {}",
                self.margin_px
            ));
        }

        if self.lost_flash_count == 0 {
            return Err("lost_flash_count must be >= 1".to_string());
        }

        if self.max_markers == 0 {
            return Err("max_markers must be >= 1".to_string());
        }

        Ok(())
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

fn validate_positive(name: &str, value: f32) -> Result<(), String> {
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("{name} must be positive, got {value}"));
    }
    Ok(())
}

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
                error!("Invalid minimap.ron reload: {reason}");
                config_state.set(ConfigState::Error(ConfigError::with_detail(
                    ConfigFile::Minimap,
                    reason,
                )));
                return;
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
        let config: MinimapConfig = ron::from_str(include_str!(
            "../../../../assets/config/base/minimap.ron"
        ))
        .expect("shipped minimap.ron parses");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_non_positive_fallback_world_radius() {
        let config = MinimapConfig {
            fallback_world_radius: 0.0,
            ..MinimapConfig::default()
        };
        let error = config.validate().expect_err("zero radius is invalid");
        assert!(error.contains("fallback_world_radius"), "{error}");
    }

    #[test]
    fn rejects_non_positive_refresh_and_scale() {
        let bad_refresh = MinimapConfig {
            refresh_secs: 0.0,
            ..MinimapConfig::default()
        };
        assert!(bad_refresh.validate().is_err());

        let bad_scale = MinimapConfig {
            radar_scale_factor: 0.0,
            ..MinimapConfig::default()
        };
        assert!(bad_scale.validate().is_err());

        let bad_flashes = MinimapConfig {
            lost_flash_count: 0,
            ..MinimapConfig::default()
        };
        assert!(bad_flashes.validate().is_err());
    }

    #[test]
    fn rejects_zero_max_markers() {
        let config = MinimapConfig {
            max_markers: 0,
            ..MinimapConfig::default()
        };
        let error = config
            .validate()
            .expect_err("zero marker capacity is invalid");
        assert!(error.contains("max_markers"), "{error}");
    }

    #[test]
    fn rejects_negative_margin() {
        let config = MinimapConfig {
            margin_px: -1.0,
            ..MinimapConfig::default()
        };
        let error = config
            .validate()
            .expect_err("negative margin is invalid");
        assert!(error.contains("margin_px"), "{error}");
    }

    #[test]
    fn scaled_world_radius_uses_radar_times_factor_or_fallback() {
        let config = MinimapConfig {
            fallback_world_radius: 6000.0,
            radar_scale_factor: 1.2,
            ..MinimapConfig::default()
        };
        assert!((config.scaled_world_radius(Some(2500.0)) - 3000.0).abs() < 1e-4);
        assert!((config.scaled_world_radius(Some(0.0)) - 6000.0).abs() < 1e-4);
        assert!((config.scaled_world_radius(None) - 6000.0).abs() < 1e-4);
    }
}
