use bevy::prelude::*;
use data::RonConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerModelConfig {
    pub model_path: String,
    pub scale: f32,
    pub y_offset: f32,
    pub yaw_offset_deg: f32,
    pub move_speed_threshold: f32,
}

impl Default for PlayerModelConfig {
    fn default() -> Self {
        Self {
            model_path: "models/player.glb".to_string(),
            scale: 0.37,
            y_offset: -0.9,
            yaw_offset_deg: 180.0,
            move_speed_threshold: 0.25,
        }
    }
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize, Default)]
pub struct VisualsConfig {
    pub player: PlayerModelConfig,
}

impl RonConfig for VisualsConfig {
    const PATH: &'static str = "config/visuals.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visuals_ron_parses_cleanly() {
        let visuals: VisualsConfig = data::assert_ron_config_parses!(VisualsConfig);
        assert!(!visuals.player.model_path.is_empty());
        assert!(visuals.player.scale > 0.0);
    }
}
