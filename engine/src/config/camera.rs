use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

/// Camera follow, aim-lead, and zoom configuration loaded from `config/camera.ron`.
#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct CameraConfig {
    pub offset: Vec3,
    pub follow_sharpness: f32,
    pub aim_lead_factor: f32,
    pub max_aim_lead: f32,
    pub zoom_default: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub zoom_step: f32,
    pub zoom_sharpness: f32,
    pub zoom_pixel_divisor: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 22.0, 12.5),
            follow_sharpness: 10.0,
            aim_lead_factor: 0.16,
            max_aim_lead: 3.5,
            zoom_default: 1.0,
            zoom_min: 0.15,
            zoom_max: 1.6,
            zoom_step: 0.1,
            zoom_sharpness: 14.0,
            zoom_pixel_divisor: 120.0,
        }
    }
}

impl RonConfig for CameraConfig {
    const PATH: &'static str = "config/camera.ron";
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn camera_ron_parses_cleanly() {
        let cfg: CameraConfig = crate::assert_ron_config_parses!(CameraConfig);
        assert!(cfg.offset.length_squared() > 0.0);
        assert!(cfg.follow_sharpness > 0.0);
        assert!(cfg.zoom_min > 0.0);
        assert!(cfg.zoom_min <= cfg.zoom_default);
        assert!(cfg.zoom_default <= cfg.zoom_max);
        assert!(cfg.zoom_step > 0.0);
        assert!(cfg.zoom_sharpness > 0.0);
        assert!(cfg.zoom_pixel_divisor > 0.0);
    }
}
