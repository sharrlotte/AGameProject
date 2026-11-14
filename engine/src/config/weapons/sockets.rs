use bevy::prelude::*;
use serde::Deserialize;

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

fn zero() -> [f32; 3] {
    [0.0; 3]
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
