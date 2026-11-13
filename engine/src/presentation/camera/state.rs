use bevy::prelude::*;

use crate::config::CameraConfig;

/// Marker component for the primary 3D gameplay camera.
#[derive(Component)]
pub struct MainCamera;

/// Smoothed zoom multiplier scaling [`CameraConfig::offset`] along the camera look ray.
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraZoom {
    pub current: f32,
    pub target: f32,
}

impl CameraZoom {
    /// Initializes both current and target zoom from the clamped default in [`CameraConfig`].
    pub fn from_config(cfg: &CameraConfig) -> Self {
        let initial = cfg.zoom_default.clamp(cfg.zoom_min, cfg.zoom_max);
        Self {
            current: initial,
            target: initial,
        }
    }

    /// Adjusts the target zoom from normalized scroll steps (positive zooms in, negative zooms out).
    pub fn adjust_target(&mut self, steps: f32, cfg: &CameraConfig) {
        self.target = (self.target - steps * cfg.zoom_step).clamp(cfg.zoom_min, cfg.zoom_max);
    }

    /// Clamps both current and target zoom to the configured range after hot-reload.
    pub fn clamp_to_config(&mut self, cfg: &CameraConfig) {
        self.target = self.target.clamp(cfg.zoom_min, cfg.zoom_max);
        self.current = self.current.clamp(cfg.zoom_min, cfg.zoom_max);
    }
}

/// Request to zoom the camera by normalized scroll steps (positive = zoom in, negative = zoom out).
#[derive(Message, Debug, Clone, Copy)]
pub struct CameraZoomMessage {
    pub steps: f32,
}







