use bevy::prelude::*;
use data::RonConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EnemyArchetypeData {
    pub max_hp: f32,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct EnemiesConfig {
    pub initial_spawn_delay: f32,
    pub spawn_interval_start: f32,
    pub spawn_interval_min: f32,
    pub escalation_duration_secs: f32,
    pub max_active_start: usize,
    pub max_active_end: usize,
    pub edge_spawn_inset: f32,
    pub min_spawn_dist_from_survivor: f32,
    pub knockback_decay_rate: f32,
    pub contact_reach_buffer: f32,
    pub hit_flash_duration: f32,
    pub walker_spawn_weight: f32,
    pub walker: EnemyArchetypeData,
    pub runner: EnemyArchetypeData,
}

impl Default for EnemiesConfig {
    fn default() -> Self {
        Self {
            initial_spawn_delay: 0.8,
            spawn_interval_start: 1.5,
            spawn_interval_min: 0.42,
            escalation_duration_secs: 240.0,
            max_active_start: 35,
            max_active_end: 80,
            edge_spawn_inset: 2.2,
            min_spawn_dist_from_survivor: 12.0,
            knockback_decay_rate: 10.0,
            contact_reach_buffer: 0.25,
            hit_flash_duration: 0.09,
            walker_spawn_weight: 0.70,
            walker: EnemyArchetypeData {
                max_hp: 75.0,
                move_speed: 4.2,
                attack_damage: 15.0,
                attack_cooldown: 0.9,
                radius: 0.45,
                half_height: 0.45,
                mass: 1.0,
            },
            runner: EnemyArchetypeData {
                max_hp: 35.0,
                move_speed: 7.8,
                attack_damage: 10.0,
                attack_cooldown: 0.7,
                radius: 0.35,
                half_height: 0.40,
                mass: 0.65,
            },
        }
    }
}

impl RonConfig for EnemiesConfig {
    const PATH: &'static str = "config/enemies/enemies.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemies_ron_parses_cleanly() {
        let enemies: EnemiesConfig = data::assert_ron_config_parses!(EnemiesConfig);
        assert!(enemies.walker.max_hp > 0.0);
    }
}
