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
    pub world_radius: f32,
    pub dot_size_px: f32,
    pub arrow_size_px: f32,
    pub max_markers: usize,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            size_px: 200.0,
            margin_px: 12.0,
            world_radius: 6000.0,
            dot_size_px: 6.0,
            arrow_size_px: 14.0,
            max_markers: 128,
        }
    }
}

impl MinimapConfig {
    pub fn validate(&self) -> Result<(), String> {
        validate_positive("size_px", self.size_px)?;
        validate_positive("world_radius", self.world_radius)?;
        validate_positive("dot_size_px", self.dot_size_px)?;
        validate_positive("arrow_size_px", self.arrow_size_px)?;

        if !self.margin_px.is_finite() || self.margin_px < 0.0 {
            return Err(format!(
                "margin_px must be non-negative, got {}",
                self.margin_px
            ));
        }

        if self.max_markers == 0 {
            return Err("max_markers must be >= 1".to_string());
        }

        Ok(())
    }

    pub fn radius_px(&self) -> f32 {
        self.size_px * 0.5
    }

    pub fn radar_ring_diameter(&self, radar_range: f32) -> f32 {
        if !radar_range.is_finite() || radar_range <= 0.0 {
            return 0.0;
        }
        (radar_range / self.world_radius) * self.size_px
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
    fn rejects_non_positive_world_radius() {
        let config = MinimapConfig {
            world_radius: 0.0,
            ..MinimapConfig::default()
        };
        let error = config.validate().expect_err("zero radius is invalid");
        assert!(error.contains("world_radius"), "{error}");
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
    fn radar_ring_scales_with_world_radius() {
        let config = MinimapConfig {
            world_radius: 6000.0,
            size_px: 200.0,
            ..MinimapConfig::default()
        };
        let diameter = config.radar_ring_diameter(2500.0);
        assert!((diameter - 200.0 * (2500.0 / 6000.0)).abs() < 1e-4);
    }
}
