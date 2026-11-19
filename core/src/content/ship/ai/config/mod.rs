pub mod types;

pub use types::*;

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::config::{ConfigError, ConfigFile, ConfigState};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AiConfig {
    pub default_enemy_level: u8,
    pub default_ally_level: u8,
    pub base_core_radar_range: f32,
    pub hysteresis: f32,
    pub investigate_arrival_radius: f32,
    pub search_patrol_radius: f32,
    pub support_orbit_factor: f32,
    pub booster_shed_supply_threshold: f32,
    pub booster_shed_battery_threshold: f32,
    pub booster_restore_battery_threshold: f32,
    pub support_auto_ratio: f32,
    #[serde(default)]
    pub bounds: IntelBounds,
    #[serde(default)]
    pub role_tuning: RoleTuning,
    #[serde(default)]
    pub board_tuning: BoardTuning,
    #[serde(default)]
    pub combat_tuning: CombatTuning,
    pub curves: SkillCurves,
    pub unlocks: SkillUnlocks,
    pub weights: UtilityWeights,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            default_enemy_level: 5,
            default_ally_level: 8,
            base_core_radar_range: 2500.0,
            hysteresis: 0.12,
            investigate_arrival_radius: 350.0,
            search_patrol_radius: 2400.0,
            support_orbit_factor: 0.75,
            booster_shed_supply_threshold: 0.75,
            booster_shed_battery_threshold: 0.25,
            booster_restore_battery_threshold: 0.55,
            support_auto_ratio: 1.0,
            bounds: IntelBounds::default(),
            role_tuning: RoleTuning::default(),
            board_tuning: BoardTuning::default(),
            combat_tuning: CombatTuning::default(),
            curves: SkillCurves {
                think_rate_hz: SkillCurve {
                    level_1: 2.0,
                    level_10: 20.0,
                },
                lead_accuracy: SkillCurve {
                    level_1: 0.15,
                    level_10: 1.0,
                },
                aim_jitter_rad: SkillCurve {
                    level_1: 0.14,
                    level_10: 0.0,
                },
                range_precision: SkillCurve {
                    level_1: 0.55,
                    level_10: 0.98,
                },
                evasion_strength: SkillCurve {
                    level_1: 0.0,
                    level_10: 1.0,
                },
                memory_secs: SkillCurve {
                    level_1: 6.0,
                    level_10: 18.0,
                },
                alert_ttl_secs: SkillCurve {
                    level_1: 5.0,
                    level_10: 12.0,
                },
            },
            unlocks: SkillUnlocks {
                omnidirectional_thrust_level: 4,
                power_management_level: 4,
                focus_fire_level: 5,
                support_coordination_level: 4,
                strafe_evade_level: 6,
                spinal_dodge_level: 7,
                armor_angling_level: 8,
            },
            weights: UtilityWeights {
                retreat_hp_threshold: 0.28,
                recharge_energy_threshold: 0.30,
                regroup_ally_distance: 2800.0,
                engage_base: 0.72,
                retreat_max: 0.98,
                recharge_max: 0.84,
                support_base: 0.78,
                regroup_base: 0.55,
                investigate_base: 0.60,
                search_base: 0.25,
            },
        }
    }
}

impl AiConfig {
    pub fn validate(&self) -> Result<(), String> {
        validate_level("default_enemy_level", self.default_enemy_level)?;
        validate_level("default_ally_level", self.default_ally_level)?;

        for (name, level) in [
            ("omnidirectional_thrust_level", self.unlocks.omnidirectional_thrust_level),
            ("power_management_level", self.unlocks.power_management_level),
            ("focus_fire_level", self.unlocks.focus_fire_level),
            ("support_coordination_level", self.unlocks.support_coordination_level),
            ("strafe_evade_level", self.unlocks.strafe_evade_level),
            ("spinal_dodge_level", self.unlocks.spinal_dodge_level),
            ("armor_angling_level", self.unlocks.armor_angling_level),
        ] {
            validate_level(name, level)?;
        }

        self.curves.think_rate_hz.validate("think_rate_hz", 0.1)?;
        self.curves.lead_accuracy.validate("lead_accuracy", 0.0)?;
        self.curves.aim_jitter_rad.validate("aim_jitter_rad", 0.0)?;
        self.curves.range_precision.validate("range_precision", 0.0)?;
        self.curves.evasion_strength.validate("evasion_strength", 0.0)?;
        self.curves.memory_secs.validate("memory_secs", 0.1)?;
        self.curves.alert_ttl_secs.validate("alert_ttl_secs", 0.1)?;

        if !self.base_core_radar_range.is_finite() || self.base_core_radar_range <= 0.0 {
            return Err("base_core_radar_range must be positive".to_string());
        }
        if !self.hysteresis.is_finite() || self.hysteresis < 0.0 {
            return Err("hysteresis must be non-negative".to_string());
        }
        if self.booster_restore_battery_threshold < self.booster_shed_battery_threshold {
            return Err(
                "booster_restore_battery_threshold must be >= booster_shed_battery_threshold"
                    .to_string(),
            );
        }
        if self.bounds.max_team_alerts == 0
            || self.bounds.max_last_known == 0
            || self.bounds.max_local_contacts == 0
        {
            return Err("intel bounds must be >= 1".to_string());
        }
        if self.combat_tuning.strafe_flip_max_secs < self.combat_tuning.strafe_flip_min_secs {
            return Err("strafe_flip_max_secs must be >= strafe_flip_min_secs".to_string());
        }
        Ok(())
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct AiConfigValue(pub AiConfig);

fn validate_level(name: &str, level: u8) -> Result<(), String> {
    if !(1..=10).contains(&level) {
        return Err(format!("{name} must be within 1..=10, got {level}"));
    }
    Ok(())
}

pub fn ai_config_bridge(
    mut events: MessageReader<AssetEvent<AiConfig>>,
    assets: Res<Assets<AiConfig>>,
    mut value: ResMut<AiConfigValue>,
    mut config_state: ResMut<NextState<ConfigState>>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
            if let Err(reason) = config.validate() {
                error!("Invalid ai.ron reload: {reason}");
                config_state.set(ConfigState::Error(ConfigError::with_detail(
                    ConfigFile::Ai,
                    reason,
                )));
                return;
            }
            *value = AiConfigValue(config.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_ai_config_parses_and_validates() {
        let config: AiConfig =
            ron::from_str(include_str!("../../../../../assets/config/base/ai.ron"))
                .expect("shipped ai.ron parses");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn skill_curve_samples_linearly() {
        let curve = SkillCurve {
            level_1: 2.0,
            level_10: 20.0,
        };
        assert!((curve.sample(1) - 2.0).abs() < 1e-5);
        assert!((curve.sample(10) - 20.0).abs() < 1e-5);
        assert!((curve.sample(5) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn rejects_invalid_ai_config() {
        let mut bad_level = AiConfig::default();
        bad_level.default_enemy_level = 11;
        assert!(bad_level.validate().is_err());

        let mut bad_rate = AiConfig::default();
        bad_rate.curves.think_rate_hz.level_1 = 0.0;
        assert!(bad_rate.validate().is_err());

        let mut bad_bounds = AiConfig::default();
        bad_bounds.bounds.max_team_alerts = 0;
        assert!(bad_bounds.validate().is_err());
    }
}
