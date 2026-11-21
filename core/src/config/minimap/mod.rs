use std::num::{NonZeroU32, NonZeroUsize};

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::validate::{NonNegativeF32, PositiveF32, non_zero_u32, non_zero_usize};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct MinimapConfig {
    pub size_px: PositiveF32,
    pub margin_px: NonNegativeF32,
    pub fallback_world_radius: PositiveF32,
    pub radar_scale_factor: PositiveF32,
    pub refresh_secs: PositiveF32,
    pub dot_size_px: PositiveF32,
    pub lost_flash_count: NonZeroU32,
    pub lost_flash_secs: PositiveF32,
    pub glow_blur_px: PositiveF32,
    pub max_markers: NonZeroUsize,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            size_px: PositiveF32::from_const(200.0),
            margin_px: NonNegativeF32::from_const(12.0),
            fallback_world_radius: PositiveF32::from_const(6000.0),
            radar_scale_factor: PositiveF32::from_const(1.2),
            refresh_secs: PositiveF32::from_const(1.0),
            dot_size_px: PositiveF32::from_const(4.0),
            lost_flash_count: non_zero_u32(3),
            lost_flash_secs: PositiveF32::from_const(0.5),
            glow_blur_px: PositiveF32::from_const(2.0),
            max_markers: non_zero_usize(128),
        }
    }
}

impl MinimapConfig {
    pub fn radius_px(&self) -> f32 {
        self.size_px.get() * 0.5
    }

    pub fn scaled_world_radius(&self, radar_range: Option<f32>) -> f32 {
        match radar_range {
            Some(range) if range.is_finite() && range > 0.0 => range * self.radar_scale_factor.get(),
            _ => self.fallback_world_radius.get(),
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct MinimapConfigValue(pub MinimapConfig);

pub fn minimap_config_bridge(
    mut events: MessageReader<AssetEvent<MinimapConfig>>,
    assets: Res<Assets<MinimapConfig>>,
    mut value: ResMut<MinimapConfigValue>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
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
        let _config: MinimapConfig =
            ron::from_str(&text).expect("minimap.ron should parse as MinimapConfig");
    }

    #[test]
    fn non_positive_size_fails_validation() {
        let ron = "(size_px: 0.0, margin_px: 12.0, fallback_world_radius: 6000.0, radar_scale_factor: 1.2, refresh_secs: 1.0, dot_size_px: 4.0, lost_flash_count: 3, lost_flash_secs: 0.5, glow_blur_px: 2.0, max_markers: 128)";
        let err = ron::from_str::<MinimapConfig>(ron).expect_err("size_px 0 must fail");
        assert!(err.to_string().contains("finite and positive"), "{err}");
    }

    #[test]
    fn zero_lost_flash_count_fails_validation() {
        let ron = "(size_px: 200.0, margin_px: 12.0, fallback_world_radius: 6000.0, radar_scale_factor: 1.2, refresh_secs: 1.0, dot_size_px: 4.0, lost_flash_count: 0, lost_flash_secs: 0.5, glow_blur_px: 2.0, max_markers: 128)";
        assert!(ron::from_str::<MinimapConfig>(ron).is_err(), "lost_flash_count 0 must fail");
    }
}
