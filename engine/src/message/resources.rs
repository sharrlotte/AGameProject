use bevy::prelude::*;
use crate::common::UpgradeKind;

#[derive(Resource, Default, Debug, Clone)]
pub struct RunStats {
    pub kills: u32,
    pub elapsed_secs: f32,
    pub run_scrap: u32,
    pub extraction_progress_secs: f32,
    pub extraction_hold_secs: f32,
    pub in_extraction_zone: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ScrapBank {
    pub banked: u32,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub enum LastRunOutcome {
    #[default]
    None,
    Extracted {
        scrap_banked: u32,
        kills: u32,
        elapsed_secs: f32,
    },
    Died {
        scrap_lost: u32,
        kills: u32,
        elapsed_secs: f32,
    },
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct UpgradeLevels {
    pub weapon_damage: u32,
    pub max_hp: u32,
    pub move_and_reload: u32,
}

impl UpgradeLevels {
    pub fn level(&self, kind: UpgradeKind) -> u32 {
        match kind {
            UpgradeKind::WeaponDamage => self.weapon_damage,
            UpgradeKind::MaxHp => self.max_hp,
            UpgradeKind::MoveAndReloadSpeed => self.move_and_reload,
        }
    }

    pub fn increment(&mut self, kind: UpgradeKind) {
        match kind {
            UpgradeKind::WeaponDamage => self.weapon_damage += 1,
            UpgradeKind::MaxHp => self.max_hp += 1,
            UpgradeKind::MoveAndReloadSpeed => self.move_and_reload += 1,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct UpgradeModifiers {
    pub damage_multiplier: f32,
    pub bonus_max_hp: f32,
    pub move_speed_multiplier: f32,
    pub reload_speed_multiplier: f32,
}

impl Default for UpgradeModifiers {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            bonus_max_hp: 0.0,
            move_speed_multiplier: 1.0,
            reload_speed_multiplier: 1.0,
        }
    }
}
