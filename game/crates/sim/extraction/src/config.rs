use bevy::prelude::*;
use data::RonConfig;
use serde::Deserialize;

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct ExtractionConfig {
    pub pad_center: Vec3,
    pub pad_radius: f32,
    pub hold_duration_secs: f32,
    pub scrap_pickup_radius: f32,
    pub scrap_magnet_radius: f32,
    pub scrap_magnet_speed: f32,
    pub scrap_radius: f32,
    pub scrap_lifetime_secs: f32,
    pub walker_scrap: u32,
    pub runner_scrap: u32,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            pad_center: Vec3::new(0.0, 0.0, -19.5),
            pad_radius: 3.2,
            hold_duration_secs: 10.0,
            scrap_pickup_radius: 1.1,
            scrap_magnet_radius: 4.2,
            scrap_magnet_speed: 11.0,
            scrap_radius: 0.25,
            scrap_lifetime_secs: 60.0,
            walker_scrap: 2,
            runner_scrap: 3,
        }
    }
}

impl RonConfig for ExtractionConfig {
    const PATH: &'static str = "config/extraction.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction_ron_parses_cleanly() {
        let extraction: ExtractionConfig = data::assert_ron_config_parses!(ExtractionConfig);
        assert!(extraction.hold_duration_secs > 0.0);
    }
}
