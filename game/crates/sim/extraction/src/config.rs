use bevy::prelude::*;
use data::{RonConfig, UpgradeKind, UpgradeLevels, UpgradeModifiers};
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

impl RonConfig for UpgradesConfig {
    const PATH: &'static str = "config/upgrades.ron";
}

/// Extension methods deriving upgrade caps and costs from [`UpgradesConfig`].
pub trait UpgradeLevelsExt {
    fn max_level(&self, kind: UpgradeKind, cfg: &UpgradesConfig) -> u32;
    fn next_cost(&self, kind: UpgradeKind, cfg: &UpgradesConfig) -> u32;
}

impl UpgradeLevelsExt for UpgradeLevels {
    fn max_level(&self, kind: UpgradeKind, cfg: &UpgradesConfig) -> u32 {
        match kind {
            UpgradeKind::WeaponDamage => cfg.damage_max_level,
            UpgradeKind::MaxHp => cfg.max_hp_max_level,
            UpgradeKind::MoveAndReloadSpeed => cfg.speed_reload_max_level,
        }
    }

    fn next_cost(&self, kind: UpgradeKind, cfg: &UpgradesConfig) -> u32 {
        let lvl = self.level(kind);
        match kind {
            UpgradeKind::WeaponDamage => cfg.damage_base_cost + lvl * cfg.damage_cost_step,
            UpgradeKind::MaxHp => cfg.max_hp_base_cost + lvl * cfg.max_hp_cost_step,
            UpgradeKind::MoveAndReloadSpeed => {
                cfg.speed_reload_base_cost + lvl * cfg.speed_reload_cost_step
            }
        }
    }
}

/// Extension method recomputing [`UpgradeModifiers`] from levels and [`UpgradesConfig`].
pub trait UpgradeModifiersExt {
    fn recompute(&mut self, levels: &UpgradeLevels, cfg: &UpgradesConfig);
}

impl UpgradeModifiersExt for UpgradeModifiers {
    fn recompute(&mut self, levels: &UpgradeLevels, cfg: &UpgradesConfig) {
        self.damage_multiplier = 1.0 + levels.weapon_damage as f32 * cfg.damage_bonus_per_level;
        self.bonus_max_hp = levels.max_hp as f32 * cfg.max_hp_bonus_per_level;
        let speed_bonus = levels.move_and_reload as f32 * cfg.speed_reload_bonus_per_level;
        self.move_speed_multiplier = 1.0 + speed_bonus;
        self.reload_speed_multiplier = 1.0 + speed_bonus;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction_ron_parses_cleanly() {
        let extraction: ExtractionConfig = data::assert_ron_config_parses!(ExtractionConfig);
        assert!(extraction.hold_duration_secs > 0.0);
    }

    #[test]
    fn upgrades_ron_parses_cleanly() {
        let upgrades: UpgradesConfig = data::assert_ron_config_parses!(UpgradesConfig);
        assert!(upgrades.damage_max_level > 0);
    }
}
