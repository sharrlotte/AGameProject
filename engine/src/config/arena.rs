use bevy::prelude::*;
use crate::config::RonConfig;
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
        }
    }
}

impl RonConfig for ArenaConfig {
    const PATH: &'static str = "config/world/arena.ron";
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn arena_ron_parses_cleanly() {
        let arena: ArenaConfig = crate::assert_ron_config_parses!(ArenaConfig);
        assert!(arena.half_size > 0.0);
    }
}
