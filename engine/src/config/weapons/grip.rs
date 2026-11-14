use bevy::prelude::*;
use serde::{Deserialize, Deserializer};

fn min_support_separation() -> f32 {
    0.08
}

fn some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// Finger and thumb curl, in degrees, applied about each bone's local X axis.
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
    pub fn is_neutral(&self) -> bool {
        self.middle_hand_deg.abs() < f32::EPSILON
            && self.fingers_deg.abs() < f32::EPSILON
            && self.thumb1_deg.abs() < f32::EPSILON
            && self.thumb2_deg.abs() < f32::EPSILON
    }
}

/// How one hand holds the weapon, expressed in weapon space.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct GripHand {
    #[serde(default, deserialize_with = "some")]
    pub position: Option<[f32; 3]>,
    #[serde(default, deserialize_with = "some")]
    pub finger: Option<[f32; 3]>,
    #[serde(default, deserialize_with = "some")]
    pub palm: Option<[f32; 3]>,
    #[serde(default, deserialize_with = "some")]
    pub elbow: Option<[f32; 3]>,
    #[serde(default)]
    pub fingers: Option<FingerCurl>,
    #[serde(default, deserialize_with = "some")]
    pub wrist_deg: Option<[f32; 3]>,
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct WeaponGripConfig {
    #[serde(default)]
    pub primary: GripHand,
    #[serde(default)]
    pub support: Option<GripHand>,
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

pub fn euler_deg(degrees: [f32; 3]) -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        degrees[0].to_radians(),
        degrees[1].to_radians(),
        degrees[2].to_radians(),
    )
    .normalize()
}
