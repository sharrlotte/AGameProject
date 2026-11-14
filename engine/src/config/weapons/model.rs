use bevy::prelude::*;
use serde::Deserialize;

fn anchor_offset() -> [f32; 3] {
    [0.0, 0.25, 0.0]
}

fn zero() -> [f32; 3] {
    [0.0; 3]
}

/// The coordinate space a weapon model is authored in.
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
