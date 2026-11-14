use bevy::prelude::*;
use crate::config::RonConfig;
use crate::common::DamageKind;
use serde::Deserialize;

use super::grip::WeaponGripConfig;
use super::model::{ModelSpace, WeaponAnchorConfig};
use super::pierce::{MAX_HITS, MIN_PIERCE_COST, PierceConfig};
use super::presentation::{WeaponPose, WeaponRecoil, WeaponReload, WeaponSway, WeaponSwap};
use super::sockets::{SocketOverride, WeaponSockets};

/// Unified weapon definition: cross-cutting tuning, gameplay stats, and
/// presentation data flattened into a single struct.
#[derive(Asset, Resource, TypePath, Debug, Clone, Default, Deserialize)]
pub struct WeaponDefinition {
    pub id: String,
    pub name: String,
    pub muzzle_offset: f32,
    pub sprint_spread_multiplier: f32,
    pub swap_cooldown_cap: f32,
    pub stance_hold_secs: f32,

    // Stats (flattened)
    pub full_auto: bool,
    pub fire_interval: f32,
    pub pellets: u8,
    pub damage: f32,
    pub spread_deg: f32,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    pub projectile_lifetime: f32,
    pub knockback: f32,
    #[serde(default)]
    pub damage_kind: DamageKind,
    #[serde(default)]
    pub pierce: PierceConfig,
    pub mag_size: u32,
    pub reload_secs: f32,

    // Presentation (flattened)
    pub relaxed: WeaponPose,
    pub aim: WeaponPose,
    pub recoil: WeaponRecoil,
    pub reload: WeaponReload,
    pub sway: WeaponSway,
    pub swap: WeaponSwap,
    pub model: String,
    #[serde(default)]
    pub model_space: ModelSpace,
    #[serde(default)]
    pub model_rotation_deg: [f32; 3],
    #[serde(default)]
    pub model_scale: f32,
    #[serde(default)]
    pub sockets: WeaponSockets,
    #[serde(default)]
    pub socket_overrides: Vec<SocketOverride>,
    #[serde(default)]
    pub grip: Option<WeaponGripConfig>,
}

impl WeaponDefinition {
    /// The transform that maps the weapon's authored model into weapon space.
    pub fn model_transform(&self) -> Transform {
        let model_rotation = Quat::from_euler(
            EulerRot::XYZ,
            self.model_rotation_deg[0].to_radians(),
            self.model_rotation_deg[1].to_radians(),
            self.model_rotation_deg[2].to_radians(),
        )
        .normalize();
        let source_rotation = self.model_space.rotation();
        Transform::from_rotation((model_rotation * source_rotation).normalize())
            .with_scale(Vec3::splat(self.model_scale))
    }
}

/// The unified weapon roster loaded from the single weapon RON asset.
#[derive(Asset, Resource, TypePath, Debug, Clone, Default, Deserialize)]
pub struct WeaponRoster {
    pub anchor: WeaponAnchorConfig,
    pub weapons: Vec<WeaponDefinition>,
}

impl WeaponRoster {
    pub fn weapon(&self, id: &str) -> Option<&WeaponDefinition> {
        self.weapons.iter().find(|weapon| weapon.id == id)
    }

    /// Validates the authored roster before any asset is mounted or fired.
    pub fn validate(&self) -> Result<(), String> {
        if self.weapons.is_empty() {
            return Err("weapon roster must define at least one weapon".to_string());
        }
        for (index, weapon) in self.weapons.iter().enumerate() {
            if weapon.id.is_empty() {
                return Err("weapon id must not be empty".to_string());
            }
            if weapon.model.is_empty() {
                return Err(format!("weapon '{}' must name a model", weapon.id));
            }
            if !weapon.model_rotation_deg.iter().all(|c| c.is_finite()) {
                return Err(format!(
                    "weapon '{}' model rotation must be finite",
                    weapon.id
                ));
            }
            if !weapon.model_scale.is_finite() || weapon.model_scale <= 0.0 {
                return Err(format!(
                    "weapon '{}' model scale must be greater than zero",
                    weapon.id
                ));
            }
            let sockets = &weapon.sockets;
            if sockets.named().iter().any(|name| name.is_empty()) {
                return Err(format!("weapon '{}' has an empty socket name", weapon.id));
            }
            for pose in [&weapon.relaxed, &weapon.aim] {
                if !pose.offset.iter().all(|c| c.is_finite())
                    || !pose.rotation_deg.iter().all(|c| c.is_finite())
                {
                    return Err(format!("weapon '{}' has a non-finite pose", weapon.id));
                }
            }
            let recoil = &weapon.recoil;
            if !recoil.pitch_deg.is_finite() || recoil.pitch_deg < 0.0 {
                return Err(format!(
                    "weapon '{}' recoil pitch must be non-negative",
                    weapon.id
                ));
            }
            if !recoil.back.is_finite() || recoil.back < 0.0 {
                return Err(format!(
                    "weapon '{}' recoil back must be non-negative",
                    weapon.id
                ));
            }
            if !recoil.max_deg.is_finite() || recoil.max_deg < recoil.pitch_deg {
                return Err(format!(
                    "weapon '{}' recoil max must be at least the impulse",
                    weapon.id
                ));
            }
            if !recoil.max_back.is_finite() || recoil.max_back < 0.0 {
                return Err(format!(
                    "weapon '{}' recoil max_back must be non-negative",
                    weapon.id
                ));
            }
            if !recoil.recover_secs.is_finite() || recoil.recover_secs <= 0.0 {
                return Err(format!(
                    "weapon '{}' recoil recover_secs must be positive",
                    weapon.id
                ));
            }
            let reload = &weapon.reload;
            if !reload.tilt_deg.is_finite()
                || !reload.drop.is_finite()
                || !reload.mag_drop.is_finite()
                || reload.mag_drop < 0.0
            {
                return Err(format!(
                    "weapon '{}' has an invalid reload pose",
                    weapon.id
                ));
            }
            if !reload.hand_return.is_finite() || !(0.0..=1.0).contains(&reload.hand_return) {
                return Err(format!(
                    "weapon '{}' reload hand_return must be in [0, 1]",
                    weapon.id
                ));
            }
            let sway = &weapon.sway;
            if !sway.amplitude.is_finite()
                || !sway.roll_deg.is_finite()
                || !sway.frequency.is_finite()
                || sway.amplitude < 0.0
                || sway.frequency <= 0.0
            {
                return Err(format!(
                    "weapon '{}' has an invalid sway config",
                    weapon.id
                ));
            }
            let swap = &weapon.swap;
            if !swap.lower_secs.is_finite()
                || !swap.raise_secs.is_finite()
                || swap.lower_secs < 0.0
                || swap.raise_secs < 0.0
                || swap.lower_secs + swap.raise_secs <= 0.0
            {
                return Err(format!(
                    "weapon '{}' has an invalid swap timeline",
                    weapon.id
                ));
            }
            let pierce = &weapon.pierce;
            if !pierce.power.is_finite() || pierce.power < 0.0 {
                return Err(format!(
                    "weapon '{}' pierce power must be non-negative and finite",
                    weapon.id
                ));
            }
            if !pierce.penalty.is_finite() || pierce.penalty < 0.0 {
                return Err(format!(
                    "weapon '{}' pierce penalty must be non-negative and finite",
                    weapon.id
                ));
            }
            let max_allowed_power = (MAX_HITS as f32) * MIN_PIERCE_COST;
            if pierce.power > max_allowed_power {
                return Err(format!(
                    "weapon '{}' pierce power exceeds max allowable ({:.2} > {:.2})",
                    weapon.id, pierce.power, max_allowed_power
                ));
            }
            if index + 1 < self.weapons.len() {
                for other in &self.weapons[index + 1..] {
                    if other.id == weapon.id {
                        return Err(format!("duplicate weapon id '{}'", weapon.id));
                    }
                }
            }
        }
        Ok(())
    }
}

impl RonConfig for WeaponRoster {
    const PATH: &'static str = "config/weapon/weapon.ron";
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn weapons_ron_parses_cleanly() {
        let roster: WeaponRoster = crate::assert_ron_config_parses!(WeaponRoster);
        assert_eq!(roster.weapons.len(), 5);
    }

    #[test]
    fn existing_weapons_default_to_ballistic_damage() {
        let roster: WeaponRoster = crate::assert_ron_config_parses!(WeaponRoster);
        assert!(
            roster
                .weapons
                .iter()
                .all(|weapon| weapon.damage_kind == DamageKind::Ballistic)
        );
    }

    #[test]
    fn weapons_ron_parses_with_valid_pierce_config() {
        let roster: WeaponRoster = crate::assert_ron_config_parses!(WeaponRoster);
        roster.validate().expect("authored weapon roster is valid");

        let m16 = roster.weapon("m16").expect("m16 exists");
        assert_eq!(m16.pierce.power, 1.6);
        assert_eq!(m16.pierce.penalty, 5.0);

        let m1911 = roster.weapon("m1911").expect("m1911 exists");
        assert_eq!(m1911.pierce.power, 0.8);
        assert_eq!(m1911.pierce.penalty, 3.0);
    }

    #[test]
    fn invalid_pierce_values_are_rejected() {
        let base_roster: WeaponRoster = crate::assert_ron_config_parses!(WeaponRoster);

        // Negative power
        let mut roster = base_roster.clone();
        roster.weapons[0].pierce.power = -0.1;
        assert!(roster.validate().is_err());

        // Negative penalty
        let mut roster = base_roster.clone();
        roster.weapons[0].pierce.penalty = -1.0;
        assert!(roster.validate().is_err());

        // Non-finite
        let mut roster = base_roster.clone();
        roster.weapons[0].pierce.power = f32::NAN;
        assert!(roster.validate().is_err());

        // Excess power above MAX_HITS * MIN_PIERCE_COST (16 * 0.25 = 4.0)
        let mut roster = base_roster.clone();
        roster.weapons[0].pierce.power = 4.01;
        let err = roster.validate().expect_err("excess power must fail");
        assert!(err.contains("exceeds max allowable"));
    }
}
