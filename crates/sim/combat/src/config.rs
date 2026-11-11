use bevy::prelude::*;
use data::RonConfig;
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
            muzzle_offset: 0.75,
            sprint_spread_multiplier: 1.65,
            swap_cooldown_cap: 0.12,
            stance_hold_secs: stance_hold(),
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

impl RonConfig for WeaponsConfig {
    const PATH: &'static str = "config/combat/weapons.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapons_ron_parses_cleanly() {
        let weapons: WeaponsConfig = data::assert_ron_config_parses!(WeaponsConfig);
        assert_eq!(weapons.weapons.len(), 2);
    }
}
