pub mod types;

pub use types::*;

use bevy::asset::AssetEvent;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::validate::{NonNegativeF32, OrderedRange, PositiveF32, SkillLevel};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AiConfig {
    pub default_enemy_level: SkillLevel,
    pub default_ally_level: SkillLevel,
    pub base_core_radar_range: PositiveF32,
    pub hysteresis: NonNegativeF32,
    pub investigate_arrival_radius: f32,
    pub search_patrol_radius: f32,
    pub support_orbit_factor: f32,
    pub booster_shed_supply_threshold: f32,
    pub booster_battery_band: OrderedRange<NonNegativeF32>,
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
            default_enemy_level: SkillLevel::from_const(5),
            default_ally_level: SkillLevel::from_const(8),
            base_core_radar_range: PositiveF32::from_const(2500.0),
            hysteresis: NonNegativeF32::from_const(0.12),
            investigate_arrival_radius: 350.0,
            search_patrol_radius: 2400.0,
            support_orbit_factor: 0.75,
            booster_shed_supply_threshold: 0.75,
            booster_battery_band: OrderedRange::<NonNegativeF32>::from_const(0.25, 0.55),
            support_auto_ratio: 1.0,
            bounds: IntelBounds::default(),
            role_tuning: RoleTuning::default(),
            board_tuning: BoardTuning::default(),
            combat_tuning: CombatTuning::default(),
            curves: SkillCurves {
                think_rate_hz: SkillCurve {
                    level_1: PositiveF32::from_const(2.0),
                    level_10: PositiveF32::from_const(20.0),
                },
                lead_accuracy: SkillCurve {
                    level_1: NonNegativeF32::from_const(0.15),
                    level_10: NonNegativeF32::from_const(1.0),
                },
                aim_jitter_rad: SkillCurve {
                    level_1: NonNegativeF32::from_const(0.14),
                    level_10: NonNegativeF32::from_const(0.0),
                },
                range_precision: SkillCurve {
                    level_1: NonNegativeF32::from_const(0.55),
                    level_10: NonNegativeF32::from_const(0.98),
                },
                evasion_strength: SkillCurve {
                    level_1: NonNegativeF32::from_const(0.0),
                    level_10: NonNegativeF32::from_const(1.0),
                },
                memory_secs: SkillCurve {
                    level_1: PositiveF32::from_const(6.0),
                    level_10: PositiveF32::from_const(18.0),
                },
                alert_ttl_secs: SkillCurve {
                    level_1: PositiveF32::from_const(5.0),
                    level_10: PositiveF32::from_const(12.0),
                },
            },
            unlocks: SkillUnlocks {
                omnidirectional_thrust_level: SkillLevel::from_const(4),
                power_management_level: SkillLevel::from_const(4),
                focus_fire_level: SkillLevel::from_const(5),
                support_coordination_level: SkillLevel::from_const(4),
                strafe_evade_level: SkillLevel::from_const(6),
                spinal_dodge_level: SkillLevel::from_const(7),
                armor_angling_level: SkillLevel::from_const(8),
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

#[derive(Resource, Default, Clone, Debug)]
pub struct AiConfigValue(pub AiConfig);

pub fn ai_config_bridge(
    mut events: MessageReader<AssetEvent<AiConfig>>,
    assets: Res<Assets<AiConfig>>,
    mut value: ResMut<AiConfigValue>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
            *value = AiConfigValue(config.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_ai_config_parses_and_validates() {
        let _config: AiConfig =
            ron::from_str(include_str!("../../../assets/config/base/ai.ron"))
                .expect("shipped ai.ron parses");
    }

    #[test]
    fn skill_curve_samples_linearly() {
        let curve = SkillCurve {
            level_1: PositiveF32::new(2.0).expect("2.0"),
            level_10: PositiveF32::new(20.0).expect("20.0"),
        };
        assert!((curve.sample(SkillLevel::MIN) - 2.0).abs() < 1e-5);
        assert!((curve.sample(SkillLevel::MAX) - 20.0).abs() < 1e-5);
        assert!((curve.sample(SkillLevel::from_const(5)) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn rejects_invalid_ai_config() {
        let base = include_str!("../../../assets/config/base/ai.ron");

        let bad_level = base.replace("default_enemy_level: 5", "default_enemy_level: 11");
        assert!(ron::from_str::<AiConfig>(&bad_level).is_err());

        let bad_rate = base.replace("think_rate_hz: (level_1: 2.0", "think_rate_hz: (level_1: 0.0");
        assert!(ron::from_str::<AiConfig>(&bad_rate).is_err());

        let bad_bounds = base.replace("max_team_alerts: 8", "max_team_alerts: 0");
        assert!(ron::from_str::<AiConfig>(&bad_bounds).is_err());

        let bad_band = base.replace("booster_battery_band: (0.25, 0.55)", "booster_battery_band: (0.6, 0.2)");
        assert!(ron::from_str::<AiConfig>(&bad_band).is_err());
    }
}
