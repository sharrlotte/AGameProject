use serde::Deserialize;

/// Pose data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponPose {
    pub offset: [f32; 3],
    pub rotation_deg: [f32; 3],
}

/// Recoil data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponRecoil {
    pub pitch_deg: f32,
    pub back: f32,
    pub recover_secs: f32,
    pub max_deg: f32,
    pub max_back: f32,
}

/// Reload data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponReload {
    pub tilt_deg: f32,
    pub drop: f32,
    pub mag_drop: f32,
    pub hand_return: f32,
}

/// Sway data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponSway {
    pub amplitude: f32,
    pub roll_deg: f32,
    pub frequency: f32,
}

/// Swap data for a unified weapon definition.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WeaponSwap {
    pub lower_secs: f32,
    pub raise_secs: f32,
    pub drop: f32,
}

