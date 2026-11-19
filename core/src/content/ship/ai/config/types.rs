use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SkillCurve {
    pub level_1: f32,
    pub level_10: f32,
}

impl SkillCurve {
    pub fn sample(self, level: u8) -> f32 {
        let t = (level.clamp(1, 10) - 1) as f32 / 9.0;
        self.level_1 + (self.level_10 - self.level_1) * t
    }

    pub fn validate(self, name: &str, min_bound: f32) -> Result<(), String> {
        if !self.level_1.is_finite() || !self.level_10.is_finite() {
            return Err(format!("{name} endpoints must be finite"));
        }
        if self.level_1 < min_bound || self.level_10 < min_bound {
            return Err(format!("{name} endpoints must be >= {min_bound}"));
        }
        Ok(())
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SkillCurves {
    pub think_rate_hz: SkillCurve,
    pub lead_accuracy: SkillCurve,
    pub aim_jitter_rad: SkillCurve,
    pub range_precision: SkillCurve,
    pub evasion_strength: SkillCurve,
    pub memory_secs: SkillCurve,
    pub alert_ttl_secs: SkillCurve,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SkillUnlocks {
    pub omnidirectional_thrust_level: u8,
    pub power_management_level: u8,
    pub focus_fire_level: u8,
    pub support_coordination_level: u8,
    pub strafe_evade_level: u8,
    pub spinal_dodge_level: u8,
    pub armor_angling_level: u8,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct UtilityWeights {
    pub retreat_hp_threshold: f32,
    pub recharge_energy_threshold: f32,
    pub regroup_ally_distance: f32,
    pub engage_base: f32,
    pub retreat_max: f32,
    pub recharge_max: f32,
    pub support_base: f32,
    pub regroup_base: f32,
    pub investigate_base: f32,
    pub search_base: f32,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct IntelBounds {
    pub max_team_alerts: usize,
    pub alert_merge_radius: f32,
    pub alert_spotted_radius: f32,
    pub max_last_known: usize,
    pub max_local_contacts: usize,
}

impl Default for IntelBounds {
    fn default() -> Self {
        Self {
            max_team_alerts: 8,
            alert_merge_radius: 450.0,
            alert_spotted_radius: 600.0,
            max_last_known: 16,
            max_local_contacts: 16,
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RoleTuning {
    pub point_defense_support_weight: f32,
    pub pulling_beam_support_weight: f32,
    pub booster_support_weight: f32,
    pub turret_optimal_range_factor: f32,
    pub sniper_fixed_dps_ratio: f32,
    pub sniper_min_optimal_range: f32,
    pub skirmisher_min_optimal_range: f32,
    pub skirmisher_min_thrust_to_mass: f32,
}

impl Default for RoleTuning {
    fn default() -> Self {
        Self {
            point_defense_support_weight: 40.0,
            pulling_beam_support_weight: 35.0,
            booster_support_weight: 45.0,
            turret_optimal_range_factor: 0.75,
            sniper_fixed_dps_ratio: 0.5,
            sniper_min_optimal_range: 4500.0,
            skirmisher_min_optimal_range: 3200.0,
            skirmisher_min_thrust_to_mass: 2.0,
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BoardTuning {
    pub focus_damage_weight: f32,
    pub focus_threat_weight: f32,
    pub focus_distance_scale: f32,
    pub focus_distance_penalty_cap: f32,
}

impl Default for BoardTuning {
    fn default() -> Self {
        Self {
            focus_damage_weight: 0.55,
            focus_threat_weight: 0.45,
            focus_distance_scale: 20000.0,
            focus_distance_penalty_cap: 0.35,
        }
    }
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CombatTuning {
    pub recharge_threat_range_factor: f32,
    pub recharge_safe_range_factor: f32,
    pub recharge_min_safe_range: f32,
    pub recharge_throttle_cap: f32,
    pub support_approach_bonus_dist: f32,
    pub support_screen_offset: f32,
    pub support_default_radius: f32,
    pub support_throttle_cap: f32,
    pub spinal_range_factor: f32,
    pub spinal_min_range: f32,
    pub kite_hp_threshold: f32,
    pub kite_min_range: f32,
    pub orbit_beam_range_factor: f32,
    pub orbit_range_factor: f32,
    pub orbit_min_range: f32,
    pub brawl_range_factor: f32,
    pub brawl_min_range: f32,
    pub strafe_flip_min_secs: f32,
    pub strafe_flip_max_secs: f32,
    pub orbit_tangent_weight: f32,
    pub brawl_tangent_weight: f32,
    pub spinal_dodge_cos_threshold: f32,
    pub spinal_dodge_impulse: f32,
    pub armor_angle_hp_ratio: f32,
    pub drain_beam_throttle_cap: f32,
    pub drain_beam_supply_threshold: f32,
    pub arrive_throttle_cap: f32,
}

impl Default for CombatTuning {
    fn default() -> Self {
        Self {
            recharge_threat_range_factor: 1.25,
            recharge_safe_range_factor: 1.15,
            recharge_min_safe_range: 3000.0,
            recharge_throttle_cap: 0.65,
            support_approach_bonus_dist: 450.0,
            support_screen_offset: 320.0,
            support_default_radius: 260.0,
            support_throttle_cap: 0.9,
            spinal_range_factor: 0.78,
            spinal_min_range: 1200.0,
            kite_hp_threshold: 0.55,
            kite_min_range: 800.0,
            orbit_beam_range_factor: 0.75,
            orbit_range_factor: 0.9,
            orbit_min_range: 600.0,
            brawl_range_factor: 0.7,
            brawl_min_range: 450.0,
            strafe_flip_min_secs: 2.5,
            strafe_flip_max_secs: 5.0,
            orbit_tangent_weight: 0.85,
            brawl_tangent_weight: 0.35,
            spinal_dodge_cos_threshold: 0.975,
            spinal_dodge_impulse: 1.15,
            armor_angle_hp_ratio: 1.25,
            drain_beam_throttle_cap: 0.25,
            drain_beam_supply_threshold: 0.95,
            arrive_throttle_cap: 0.85,
        }
    }
}
