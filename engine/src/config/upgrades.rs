use bevy::prelude::*;
use crate::config::RonConfig;
use crate::common::UpgradeKind;
use crate::message::resources::{UpgradeLevels, UpgradeModifiers};
use serde::Deserialize;

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
    const PATH: &'static str = "config/extraction/upgrades.ron";
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
    fn upgrades_ron_parses_cleanly() {
        let upgrades: UpgradesConfig = crate::assert_ron_config_parses!(UpgradesConfig);
        assert!(upgrades.damage_max_level > 0);
    }
}

