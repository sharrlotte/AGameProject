use bevy::prelude::*;
use serde::{Deserialize, Deserializer};

fn min_support_separation() -> f32 {
    0.08
}

/// Accepts a bare value for an optional field, so authors can write
/// `finger: (0.0, -1.0, 0.0)` instead of `finger: Some((0.0, -1.0, 0.0))`.
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
            if let Some(value) = value
                && !value.iter().all(|component| component.is_finite())
            {
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
            if let Some(value) = value
                && value.length_squared() < 1e-6
            {
                return Err(format!("weapon '{weapon}' grip {label}.{name} has no length"));
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
                return Err(format!("weapon '{weapon}' grip {label}.fingers must be finite"));
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
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
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

impl Default for WeaponGripConfig {
    fn default() -> Self {
        Self {
            primary: GripHand::default(),
            support: None,
            min_support_separation: min_support_separation(),
        }
    }
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
