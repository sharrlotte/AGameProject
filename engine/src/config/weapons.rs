use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

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
    /// Seconds the combat stance stays raised after aim, fire, and reload all end.
    #[serde(default = "stance_hold")]
    pub stance_hold_secs: f32,
    pub weapons: Vec<WeaponSpecData>,
}

fn stance_hold() -> f32 {
    0.15
}

impl Default for WeaponsConfig {
    fn default() -> Self {
        Self {
            muzzle_offset: 0.6,
            sprint_spread_multiplier: 1.65,
            swap_cooldown_cap: 0.12,
            stance_hold_secs: stance_hold(),
            weapons: vec![
                WeaponSpecData {
                    id: "m16".to_string(),
                    name: "M16".to_string(),
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
                    id: "m1911".to_string(),
                    name: "M1911".to_string(),
                    full_auto: false,
                    fire_interval: 0.18,
                    pellets: 1,
                    damage: 32.0,
                    spread_deg: 2.0,
                    projectile_speed: 50.0,
                    projectile_radius: 0.08,
                    projectile_lifetime: 1.2,
                    knockback: 4.0,
                    mag_size: 7,
                    reload_secs: 1.6,
                },
                WeaponSpecData {
                    id: "mp5".to_string(),
                    name: "MP5".to_string(),
                    full_auto: true,
                    fire_interval: 0.08,
                    pellets: 1,
                    damage: 16.0,
                    spread_deg: 4.0,
                    projectile_speed: 38.0,
                    projectile_radius: 0.08,
                    projectile_lifetime: 1.0,
                    knockback: 2.0,
                    mag_size: 30,
                    reload_secs: 1.7,
                },
                WeaponSpecData {
                    id: "mp7".to_string(),
                    name: "MP7".to_string(),
                    full_auto: true,
                    fire_interval: 0.07,
                    pellets: 1,
                    damage: 14.0,
                    spread_deg: 4.5,
                    projectile_speed: 40.0,
                    projectile_radius: 0.07,
                    projectile_lifetime: 1.0,
                    knockback: 1.8,
                    mag_size: 40,
                    reload_secs: 1.9,
                },
                WeaponSpecData {
                    id: "scar".to_string(),
                    name: "SCAR".to_string(),
                    full_auto: true,
                    fire_interval: 0.12,
                    pellets: 1,
                    damage: 28.0,
                    spread_deg: 2.5,
                    projectile_speed: 45.0,
                    projectile_radius: 0.10,
                    projectile_lifetime: 1.3,
                    knockback: 3.0,
                    mag_size: 20,
                    reload_secs: 2.0,
                },
            ],
        }
    }
}

impl RonConfig for WeaponsConfig {
    const PATH: &'static str = "config/combat/weapons.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapons_ron_parses_cleanly() {
        let weapons: WeaponsConfig = crate::assert_ron_config_parses!(WeaponsConfig);
        assert_eq!(weapons.weapons.len(), 5);
    }
}

