use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

use super::model::WeaponAnchorConfig;
use super::presentation::WeaponPresentation;
use super::stats::WeaponStats;

/// Unified weapon definition: cross-cutting tuning, gameplay stats, and
/// presentation data.
#[derive(Asset, Resource, TypePath, Debug, Clone, Default, Deserialize)]
pub struct WeaponDefinition {
    pub id: String,
    pub name: String,
    pub muzzle_offset: f32,
    pub sprint_spread_multiplier: f32,
    pub swap_cooldown_cap: f32,
    pub stance_hold_secs: f32,
    pub stats: WeaponStats,
    pub presentation: WeaponPresentation,
}

impl WeaponDefinition {
    /// The transform that maps the weapon's authored model into weapon space.
    pub fn model_transform(&self) -> Transform {
        let model_rotation = Quat::from_euler(
            EulerRot::XYZ,
            self.presentation.model_rotation_deg[0].to_radians(),
            self.presentation.model_rotation_deg[1].to_radians(),
            self.presentation.model_rotation_deg[2].to_radians(),
        )
        .normalize();
        let source_rotation = self.presentation.model_space.rotation();
        Transform::from_rotation((model_rotation * source_rotation).normalize()).with_scale(
            Vec3::splat(self.presentation.model_scale),
        )
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
            if weapon.presentation.model.is_empty() {
                return Err(format!("weapon '{}' must name a model", weapon.id));
            }
            let presentation = &weapon.presentation;
            if !presentation.model_rotation_deg.iter().all(|c| c.is_finite()) {
                return Err(format!(
                    "weapon '{}' model rotation must be finite",
                    weapon.id
                ));
            }
            if !presentation.model_scale.is_finite() || presentation.model_scale <= 0.0 {
                return Err(format!(
                    "weapon '{}' model scale must be greater than zero",
                    weapon.id
                ));
            }
            let sockets = &presentation.sockets;
            if sockets.named().iter().any(|name| name.is_empty()) {
                return Err(format!("weapon '{}' has an empty socket name", weapon.id));
            }
            for pose in [&presentation.relaxed, &presentation.aim] {
                if !pose.offset.iter().all(|c| c.is_finite())
                    || !pose.rotation_deg.iter().all(|c| c.is_finite())
                {
                    return Err(format!("weapon '{}' has a non-finite pose", weapon.id));
                }
            }
            let recoil = &presentation.recoil;
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
            let reload = &presentation.reload;
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
            let sway = &presentation.sway;
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
            let swap = &presentation.swap;
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
}
