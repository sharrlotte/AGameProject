use serde::Deserialize;

use super::grip::WeaponGripConfig;
use super::model::ModelSpace;
use super::sockets::{SocketOverride, WeaponSockets};

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
