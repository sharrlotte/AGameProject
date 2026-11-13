use bevy::prelude::*;
use crate::config::RonConfig;
use serde::{Deserialize, Deserializer};

fn min_support_separation() -> f32 {
    0.08
}

fn anchor_offset() -> [f32; 3] {
    [0.0, 0.25, 0.0]
}

fn some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// Finger and thumb curl, in degrees, applied about each bone's local X axis.
///
/// The player skeleton folds its fingers around the local X axis, so a curl
/// value of zero leaves the hand flat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct FingerCurl {
    #[serde(default)]
    pub middle_hand_deg: f32,
    #[serde(default)]
    pub fingers_deg: f32,
    #[serde(default)]
    pub thumb1_deg: f32,
    #[serde(default)]
    pub thumb2_deg: f32,
}

impl FingerCurl {
    /// True when every bone keeps its animated pose.
    pub fn is_neutral(&self) -> bool {
        self.middle_hand_deg.abs() < f32::EPSILON
            && self.fingers_deg.abs() < f32::EPSILON
            && self.thumb1_deg.abs() < f32::EPSILON
            && self.thumb2_deg.abs() < f32::EPSILON
    }
}

/// How one hand holds the weapon, expressed in weapon space.
///
/// Weapon space is `+Y` up, `-Z` towards the muzzle and `+X` towards the
/// weapon's right-hand side, which matches the character's own axes.
///
/// Every field is optional. An authored value replaces the socket-derived
/// value for that field only, so a weapon can tune its palm position without
/// restating its finger direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct GripHand {
    /// Where the palm bone origin sits, relative to the weapon origin.
    #[serde(default, deserialize_with = "some")]
    pub position: Option<[f32; 3]>,
    /// Direction the fingers point, away from the wrist.
    #[serde(default, deserialize_with = "some")]
    pub finger: Option<[f32; 3]>,
    /// Direction the palm faces, pressing into the grip surface.
    #[serde(default, deserialize_with = "some")]
    pub palm: Option<[f32; 3]>,
    /// Direction the elbow is allowed to bend towards.
    #[serde(default, deserialize_with = "some")]
    pub elbow: Option<[f32; 3]>,
    /// Finger curl for this hand. Falls back to the animation catalog default.
    #[serde(default)]
    pub fingers: Option<FingerCurl>,
    /// Extra rotation in the hand bone's own frame, in degrees.
    ///
    /// Falls back to the per-hand offset in `IkConfig`.
    #[serde(default, deserialize_with = "some")]
    pub wrist_deg: Option<[f32; 3]>,
    /// Extra rotation in weapon space applied after the palm frame, in degrees.
    ///
    /// Use this to tilt a hand around the grip without changing its finger or
    /// palm directions.
    #[serde(default, deserialize_with = "some")]
    pub align_deg: Option<[f32; 3]>,
}

impl GripHand {
    pub fn position(&self) -> Option<Vec3> {
        self.position.map(Vec3::from_array)
    }

    pub fn finger(&self) -> Option<Vec3> {
        self.finger.map(Vec3::from_array)
    }

    pub fn palm(&self) -> Option<Vec3> {
        self.palm.map(Vec3::from_array)
    }

    pub fn elbow(&self) -> Option<Vec3> {
        self.elbow.map(Vec3::from_array)
    }

    pub fn wrist(&self) -> Option<Quat> {
        self.wrist_deg.map(euler_deg)
    }

    pub fn align(&self) -> Option<Quat> {
        self.align_deg.map(euler_deg)
    }

    /// Validates the authored fields.
    pub fn validate(&self, label: &str, weapon: &str) -> Result<(), String> {
        for (name, value) in [
            ("position", self.position),
            ("finger", self.finger),
            ("palm", self.palm),
            ("elbow", self.elbow),
            ("wrist_deg", self.wrist_deg),
            ("align_deg", self.align_deg),
        ] {
            if let Some(value) = value && !value.iter().all(|component| component.is_finite()) {
                return Err(format!(
                    "weapon '{weapon}' grip {label}.{name} must be finite"
                ));
            }
        }
        for (name, value) in [
            ("finger", self.finger()),
            ("palm", self.palm()),
            ("elbow", self.elbow()),
        ] {
            if let Some(value) = value && value.length_squared() < 1e-6 {
                return Err(format!(
                    "weapon '{weapon}' grip {label}.{name} has no length"
                ));
            }
        }
        if let Some(curl) = self.fingers {
            let values = [
                curl.middle_hand_deg,
                curl.fingers_deg,
                curl.thumb1_deg,
                curl.thumb2_deg,
            ];
            if !values.iter().all(|value| value.is_finite()) {
                return Err(format!(
                    "weapon '{weapon}' grip {label}.fingers must be finite"
                ));
            }
            if values.iter().any(|value| value.abs() > 120.0) {
                return Err(format!(
                    "weapon '{weapon}' grip {label}.fingers must stay within 120 degrees"
                ));
            }
        }
        Ok(())
    }
}

/// A weapon's complete grip configuration.
///
/// The primary hand holds the firing grip. `support` is `None` for a one-handed
/// weapon; when present, its omitted fields are derived from the foregrip
/// socket.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct WeaponGripConfig {
    #[serde(default)]
    pub primary: GripHand,
    #[serde(default)]
    pub support: Option<GripHand>,
    /// Minimum distance between the grip and foregrip sockets before a support
    /// hand is derived automatically.
    #[serde(default = "min_support_separation")]
    pub min_support_separation: f32,
}

impl WeaponGripConfig {
    pub fn validate(&self, weapon: &str) -> Result<(), String> {
        self.primary.validate("primary", weapon)?;
        if let Some(support) = self.support {
            support.validate("support", weapon)?;
        }
        if !self.min_support_separation.is_finite() || self.min_support_separation < 0.0 {
            return Err(format!(
                "weapon '{weapon}' grip min_support_separation must be non-negative"
            ));
        }
        Ok(())
    }
}

pub(crate) fn euler_deg(degrees: [f32; 3]) -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        degrees[0].to_radians(),
        degrees[1].to_radians(),
        degrees[2].to_radians(),
    )
    .normalize()
}

/// Socket node names authored in the weapon glTF.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponSockets {
    #[serde(default = "grip")]
    pub grip: String,
    #[serde(default = "foregrip")]
    pub foregrip: String,
    #[serde(default = "magazine")]
    pub magazine: String,
    #[serde(default = "muzzle")]
    pub muzzle: String,
}

fn grip() -> String {
    "Grip".to_string()
}

fn foregrip() -> String {
    "Foregrip".to_string()
}

fn magazine() -> String {
    "Magazine".to_string()
}

fn muzzle() -> String {
    "Muzzle".to_string()
}

impl WeaponSockets {
    pub fn named(&self) -> [&str; 4] {
        [
            self.grip.as_str(),
            self.foregrip.as_str(),
            self.magazine.as_str(),
            self.muzzle.as_str(),
        ]
    }
}

/// Replaces one socket node transform for assets that cannot be re-exported.
///
/// The values are absolute: the socket node is placed at `offset` with
/// `rotation_deg` inside the weapon model's own space.
#[derive(Debug, Clone, Deserialize)]
pub struct SocketOverride {
    pub socket: String,
    #[serde(default = "zero")]
    pub offset: [f32; 3],
    #[serde(default = "zero")]
    pub rotation_deg: [f32; 3],
}

impl SocketOverride {
    pub fn transform(&self) -> Transform {
        Transform::from_translation(Vec3::from_array(self.offset)).with_rotation(
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation_deg[0].to_radians(),
                self.rotation_deg[1].to_radians(),
                self.rotation_deg[2].to_radians(),
            )
            .normalize(),
        )
    }
}

fn zero() -> [f32; 3] {
    [0.0; 3]
}

/// The coordinate space a weapon model is authored in.
///
/// Converted FBX assets use `-Y` forward and `+Z` up, which is the Blender
/// convention before the glTF Y-up conversion. Weapon space uses `-Z` forward
/// and `+Y` up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub enum ModelSpace {
    /// Already matches weapon space: `-Z` forward, `+Y` up.
    #[default]
    Weapon,
    /// `-Y` forward, `+Z` up.
    ForwardNegYUpZ,
}

impl ModelSpace {
    /// The rotation that maps the model into weapon space.
    pub fn rotation(self) -> Quat {
        match self {
            Self::Weapon => Quat::IDENTITY,
            Self::ForwardNegYUpZ => Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
                0.0,
            )
            .normalize(),
        }
    }
}

/// Gameplay statistics for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponStats {
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

/// Presentation pose data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentationPose {
    pub offset: [f32; 3],
    pub rotation_deg: [f32; 3],
}

/// Presentation recoil data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentationRecoil {
    pub pitch_deg: f32,
    pub back: f32,
    pub recover_secs: f32,
    pub max_deg: f32,
    pub max_back: f32,
}

/// Presentation reload data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentationReload {
    pub tilt_deg: f32,
    pub drop: f32,
    pub mag_drop: f32,
    pub hand_return: f32,
}

/// Presentation sway data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentationSway {
    pub amplitude: f32,
    pub roll_deg: f32,
    pub frequency: f32,
}

/// Presentation swap data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentationSwap {
    pub lower_secs: f32,
    pub raise_secs: f32,
    pub drop: f32,
}

/// Presentation data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPresentation {
    pub relaxed: WeaponPresentationPose,
    pub aim: WeaponPresentationPose,
    pub recoil: WeaponPresentationRecoil,
    pub reload: WeaponPresentationReload,
    pub sway: WeaponPresentationSway,
    pub swap: WeaponPresentationSwap,
    /// Path of the weapon's authored model, relative to the asset root.
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

/// Chest-height mount point under the player's visual root.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponAnchorConfig {
    #[serde(default = "anchor_offset")]
    pub offset: [f32; 3],
    #[serde(default = "zero")]
    pub rotation_deg: [f32; 3],
}

impl WeaponAnchorConfig {
    pub fn offset(&self) -> Vec3 {
        Vec3::from_array(self.offset)
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
}

impl WeaponRoster {
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
