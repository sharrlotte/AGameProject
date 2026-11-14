use bevy::prelude::*;
use serde::Deserialize;

fn zero_vec() -> [f32; 3] {
    [0.0; 3]
}

/// A spatial reference point on a weapon in weapon space.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SocketPoint {
    #[serde(default = "zero_vec")]
    pub offset: [f32; 3],
    #[serde(default)]
    pub rotation_deg: Option<[f32; 3]>,
}

impl Default for SocketPoint {
    fn default() -> Self {
        Self {
            offset: [0.0; 3],
            rotation_deg: None,
        }
    }
}

impl SocketPoint {
    pub fn new(offset: [f32; 3]) -> Self {
        Self {
            offset,
            rotation_deg: None,
        }
    }

    pub fn translation(&self) -> Vec3 {
        Vec3::from_array(self.offset)
    }

    pub fn rotation(&self) -> Quat {
        match self.rotation_deg {
            Some(deg) => Quat::from_euler(
                EulerRot::XYZ,
                deg[0].to_radians(),
                deg[1].to_radians(),
                deg[2].to_radians(),
            )
            .normalize(),
            None => Quat::IDENTITY,
        }
    }

    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.translation()).with_rotation(self.rotation())
    }
}

/// Mechanical weapon reference points (such as reload reach target and muzzle origin).
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct WeaponMechanicalSockets {
    #[serde(default)]
    pub magazine: SocketPoint,
    #[serde(default)]
    pub muzzle: SocketPoint,
}

pub type WeaponSockets = WeaponMechanicalSockets;
pub type SocketOverride = SocketPoint;

