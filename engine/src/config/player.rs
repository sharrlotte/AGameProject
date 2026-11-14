use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct PlayerConfig {
    pub aim_height: f32,
    pub radius: f32,
    pub half_height: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub max_stamina: f32,
    pub stamina_drain_per_sec: f32,
    pub stamina_regen_per_sec: f32,
    pub stamina_exhaustion_delay: f32,
    pub max_hp: f32,
    pub knockback_decay_rate: f32,
    pub knockback_max_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            aim_height: 0.9,
            radius: 0.45,
            half_height: 0.45,
            walk_speed: 7.5,
            sprint_speed: 11.0,
            max_stamina: 100.0,
            stamina_drain_per_sec: 28.0,
            stamina_regen_per_sec: 22.0,
            stamina_exhaustion_delay: 1.0,
            max_hp: 100.0,
            knockback_decay_rate: 12.0,
            knockback_max_speed: 12.0,
        }
    }
}

impl RonConfig for PlayerConfig {
    const PATH: &'static str = "config/player/player.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_ron_parses_cleanly() {
        let player: PlayerConfig = crate::assert_ron_config_parses!(PlayerConfig);
        assert!(player.max_hp > 0.0);
    }
}
