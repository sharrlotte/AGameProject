use bevy::prelude::*;
use data::RonConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ObstacleDef {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub is_wall: bool,
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct ArenaConfig {
    pub half_size: f32,
    pub wall_height: f32,
    pub wall_thickness: f32,
    pub obstacles: Vec<ObstacleDef>,
    pub sun_illuminance: f32,
    pub ambient_brightness: f32,
    pub camera_offset: Vec3,
    pub camera_follow_sharpness: f32,
    pub camera_aim_lead_factor: f32,
    pub camera_max_aim_lead: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            half_size: 25.0,
            wall_height: 2.6,
            wall_thickness: 1.0,
            obstacles: Vec::new(),
            sun_illuminance: 14_000.0,
            ambient_brightness: 280.0,
            camera_offset: Vec3::new(0.0, 22.0, 12.5),
            camera_follow_sharpness: 10.0,
            camera_aim_lead_factor: 0.16,
            camera_max_aim_lead: 3.5,
        }
    }
}

impl RonConfig for ArenaConfig {
    const PATH: &'static str = "config/world/arena.ron";
}

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
    fn arena_ron_parses_cleanly() {
        let arena: ArenaConfig = data::assert_ron_config_parses!(ArenaConfig);
        assert!(arena.half_size > 0.0);
    }

    #[test]
    fn player_ron_parses_cleanly() {
        let player: PlayerConfig = data::assert_ron_config_parses!(PlayerConfig);
        assert!(player.max_hp > 0.0);
    }
}
