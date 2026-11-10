use bevy::prelude::*;
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

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponSpecData {
    pub id: String,
    pub name: String,
    pub full_auto: bool,
    pub fire_interval: f32,
    pub pellets: u8,
    pub damage: f32,
    pub spread_deg: f32,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    pub projectile_lifetime: f32,
    pub knockback: f32,
    pub mag_size: u32,
    pub reload_secs: f32,
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct WeaponsConfig {
    pub muzzle_offset: f32,
    pub sprint_spread_multiplier: f32,
    pub swap_cooldown_cap: f32,
    pub weapons: Vec<WeaponSpecData>,
}

impl Default for WeaponsConfig {
    fn default() -> Self {
        Self {
            muzzle_offset: 0.75,
            sprint_spread_multiplier: 1.65,
            swap_cooldown_cap: 0.12,
            weapons: vec![
                WeaponSpecData {
                    id: "assault_rifle".to_string(),
                    name: "Assault Rifle".to_string(),
                    full_auto: true,
                    fire_interval: 0.10,
                    pellets: 1,
                    damage: 24.0,
                    spread_deg: 3.0,
                    projectile_speed: 42.0,
                    projectile_radius: 0.10,
                    projectile_lifetime: 1.2,
                    knockback: 3.5,
                    mag_size: 30,
                    reload_secs: 1.8,
                },
                WeaponSpecData {
                    id: "shotgun".to_string(),
                    name: "Shotgun".to_string(),
                    full_auto: false,
                    fire_interval: 0.75,
                    pellets: 8,
                    damage: 14.0,
                    spread_deg: 12.0,
                    projectile_speed: 34.0,
                    projectile_radius: 0.08,
                    projectile_lifetime: 0.55,
                    knockback: 5.5,
                    mag_size: 8,
                    reload_secs: 2.2,
                },
            ],
        }
    }
}

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

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct UpgradesConfig {
    pub damage_bonus_per_level: f32,
    pub damage_base_cost: u32,
    pub damage_cost_step: u32,
    pub damage_max_level: u32,
    pub max_hp_bonus_per_level: f32,
    pub max_hp_base_cost: u32,
    pub max_hp_cost_step: u32,
    pub max_hp_max_level: u32,
    pub speed_reload_bonus_per_level: f32,
    pub speed_reload_base_cost: u32,
    pub speed_reload_cost_step: u32,
    pub speed_reload_max_level: u32,
}

impl Default for UpgradesConfig {
    fn default() -> Self {
        Self {
            damage_bonus_per_level: 0.15,
            damage_base_cost: 10,
            damage_cost_step: 10,
            damage_max_level: 10,
            max_hp_bonus_per_level: 25.0,
            max_hp_base_cost: 10,
            max_hp_cost_step: 10,
            max_hp_max_level: 10,
            speed_reload_bonus_per_level: 0.10,
            speed_reload_base_cost: 12,
            speed_reload_cost_step: 12,
            speed_reload_max_level: 8,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerModelConfig {
    pub model_path: String,
    pub scale: f32,
    pub y_offset: f32,
    pub yaw_offset_deg: f32,
    pub blend_secs: f32,
    pub walk_speed_ref: f32,
    pub sprint_playback_speed: f32,
    pub move_speed_threshold: f32,
    pub idle_clip: String,
    pub walk_clip: String,
    pub run_clip: String,
    pub death_clip: String,
}

impl Default for PlayerModelConfig {
    fn default() -> Self {
        Self {
            model_path: "models/player.glb".to_string(),
            scale: 0.37,
            y_offset: -0.9,
            yaw_offset_deg: 180.0,
            blend_secs: 0.15,
            walk_speed_ref: 7.5,
            sprint_playback_speed: 1.35,
            move_speed_threshold: 0.25,
            idle_clip: "HumanArmature|Man_Idle".to_string(),
            walk_clip: "HumanArmature|Man_Walk".to_string(),
            run_clip: "HumanArmature|Man_Run".to_string(),
            death_clip: "HumanArmature|Man_Death".to_string(),
        }
    }
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize, Default)]
pub struct VisualsConfig {
    pub player: PlayerModelConfig,
}
