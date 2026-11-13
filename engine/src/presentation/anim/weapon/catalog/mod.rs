use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

use crate::presentation::anim::grip::WeaponGripConfig;

fn zero() -> [f32; 3] {
    [0.0; 3]
}

fn one() -> f32 {
    1.0
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

/// A weapon-space offset and Euler rotation in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct WeaponPose {
    #[serde(default = "zero")]
    pub offset: [f32; 3],
    #[serde(default = "zero")]
    pub rotation_deg: [f32; 3],
}

impl Default for WeaponPose {
    fn default() -> Self {
        Self {
            offset: zero(),
            rotation_deg: zero(),
        }
    }
}

/// Socket node names authored in the weapon glTF.
#[derive(Debug, Clone, Deserialize)]
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

impl Default for WeaponSockets {
    fn default() -> Self {
        Self {
            grip: grip(),
            foregrip: foregrip(),
            magazine: magazine(),
            muzzle: muzzle(),
        }
    }
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

fn recoil_pitch() -> f32 {
    4.0
}

fn recoil_back() -> f32 {
    0.04
}

fn recoil_recover() -> f32 {
    0.12
}

fn recoil_max() -> f32 {
    12.0
}

fn recoil_max_back() -> f32 {
    0.12
}

/// Impulse and decay for a single shot.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RecoilConfig {
    #[serde(default = "recoil_pitch")]
    pub pitch_deg: f32,
    #[serde(default = "recoil_back")]
    pub back: f32,
    #[serde(default = "recoil_recover")]
    pub recover_secs: f32,
    #[serde(default = "recoil_max")]
    pub max_deg: f32,
    #[serde(default = "recoil_max_back")]
    pub max_back: f32,
}

impl Default for RecoilConfig {
    fn default() -> Self {
        Self {
            pitch_deg: recoil_pitch(),
            back: recoil_back(),
            recover_secs: recoil_recover(),
            max_deg: recoil_max(),
            max_back: recoil_max_back(),
        }
    }
}

fn reload_tilt() -> f32 {
    22.0
}

fn reload_drop() -> f32 {
    0.08
}

fn reload_mag_drop() -> f32 {
    0.12
}

fn reload_return() -> f32 {
    0.7
}

/// Procedural reload pose driven by normalized reload progress.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ReloadConfig {
    #[serde(default = "reload_tilt")]
    pub tilt_deg: f32,
    #[serde(default = "reload_drop")]
    pub drop: f32,
    #[serde(default = "reload_mag_drop")]
    pub mag_drop: f32,
    /// Progress at which the support hand returns from the magazine to the foregrip.
    #[serde(default = "reload_return")]
    pub hand_return: f32,
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            tilt_deg: reload_tilt(),
            drop: reload_drop(),
            mag_drop: reload_mag_drop(),
            hand_return: reload_return(),
        }
    }
}

fn sway_amplitude() -> f32 {
    0.012
}

fn sway_roll_deg() -> f32 {
    1.5
}

fn sway_frequency() -> f32 {
    1.4
}

/// Low-frequency weapon bob that keeps a welded weapon from looking static.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SwayConfig {
    #[serde(default = "sway_amplitude")]
    pub amplitude: f32,
    #[serde(default = "sway_roll_deg")]
    pub roll_deg: f32,
    #[serde(default = "sway_frequency")]
    pub frequency: f32,
}

impl Default for SwayConfig {
    fn default() -> Self {
        Self {
            amplitude: sway_amplitude(),
            roll_deg: sway_roll_deg(),
            frequency: sway_frequency(),
        }
    }
}

fn swap_lower() -> f32 {
    0.12
}

fn swap_raise() -> f32 {
    0.18
}

fn swap_drop() -> f32 {
    0.2
}

/// Lower and raise timing when switching weapons.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SwapConfig {
    #[serde(default = "swap_lower")]
    pub lower_secs: f32,
    #[serde(default = "swap_raise")]
    pub raise_secs: f32,
    #[serde(default = "swap_drop")]
    pub drop: f32,
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self {
            lower_secs: swap_lower(),
            raise_secs: swap_raise(),
            drop: swap_drop(),
        }
    }
}

impl SwapConfig {
    pub fn total_secs(&self) -> f32 {
        self.lower_secs.max(0.0) + self.raise_secs.max(0.0)
    }
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
    /// The rotation that maps the model space into weapon space.
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

/// One weapon's presentation data.
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponProfile {
    pub id: String,
    pub model: String,
    /// The space the model is authored in.
    #[serde(default)]
    pub model_space: ModelSpace,
    /// Extra rotation in weapon space, applied after `model_space`.
    #[serde(default = "zero")]
    pub model_rotation_deg: [f32; 3],
    /// Uniform scale applied to the model.
    #[serde(default = "one")]
    pub model_scale: f32,
    #[serde(default)]
    pub sockets: WeaponSockets,
    #[serde(default)]
    pub relaxed: WeaponPose,
    #[serde(default)]
    pub aim: WeaponPose,
    #[serde(default)]
    pub recoil: RecoilConfig,
    #[serde(default)]
    pub reload: ReloadConfig,
    #[serde(default)]
    pub sway: SwayConfig,
    #[serde(default)]
    pub swap: SwapConfig,
    #[serde(default)]
    pub socket_overrides: Vec<SocketOverride>,
    /// How the character holds this weapon. Falls back to a grip derived from
    /// the weapon's sockets when it is not authored.
    #[serde(default)]
    pub grip: Option<WeaponGripConfig>,
}

impl WeaponProfile {
    pub fn socket_override(&self, socket: &str) -> Option<&SocketOverride> {
        self.socket_overrides
            .iter()
            .find(|over| over.socket == socket)
    }

    /// The transform that maps the model into weapon space.
    pub fn model_transform(&self) -> Transform {
        let extra = Quat::from_euler(
            EulerRot::XYZ,
            self.model_rotation_deg[0].to_radians(),
            self.model_rotation_deg[1].to_radians(),
            self.model_rotation_deg[2].to_radians(),
        )
        .normalize();
        Transform::from_rotation((extra * self.model_space.rotation()).normalize())
            .with_scale(Vec3::splat(self.model_scale))
    }
}

impl Default for WeaponProfile {
    fn default() -> Self {
        Self {
            id: "assault_rifle".to_string(),
            model: "models/weapons/rifle.glb".to_string(),
            model_space: ModelSpace::Weapon,
            model_rotation_deg: zero(),
            model_scale: 1.0,
            sockets: WeaponSockets::default(),
            relaxed: WeaponPose {
                offset: [0.1, -0.25, 0.05],
                rotation_deg: [-30.0, 0.0, 0.0],
            },
            aim: WeaponPose {
                offset: [0.1, 0.0, -0.2],
                rotation_deg: [0.0, 0.0, 0.0],
            },
            recoil: RecoilConfig::default(),
            reload: ReloadConfig::default(),
            sway: SwayConfig::default(),
            swap: SwapConfig::default(),
            socket_overrides: Vec::new(),
            grip: None,
        }
    }
}

fn anchor_offset() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}

/// Chest-height mount point under `PlayerVisualRoot`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct WeaponAnchorConfig {
    #[serde(default = "anchor_offset")]
    pub offset: [f32; 3],
    #[serde(default = "zero")]
    pub rotation_deg: [f32; 3],
}

impl Default for WeaponAnchorConfig {
    fn default() -> Self {
        Self {
            offset: anchor_offset(),
            rotation_deg: zero(),
        }
    }
}

/// Hot-reloadable weapon presentation catalog.
#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct WeaponCatalog {
    #[serde(default)]
    pub anchor: WeaponAnchorConfig,
    #[serde(default = "default_weapons")]
    pub weapons: Vec<WeaponProfile>,
}

fn default_weapons() -> Vec<WeaponProfile> {
    vec![WeaponProfile::default()]
}

impl Default for WeaponCatalog {
    fn default() -> Self {
        Self {
            anchor: WeaponAnchorConfig::default(),
            weapons: default_weapons(),
        }
    }
}

impl WeaponCatalog {
    pub fn weapon(&self, id: &str) -> Option<&WeaponProfile> {
        self.weapons.iter().find(|weapon| weapon.id == id)
    }

    /// Validates profiles and socket names before any asset is mounted.
    pub fn validate(&self) -> Result<(), String> {
        if self.weapons.is_empty() {
            return Err("weapon catalog must define at least one weapon".to_string());
        }
        if !finite3(self.anchor.offset) || !finite3(self.anchor.rotation_deg) {
            return Err("weapon anchor offsets must be finite".to_string());
        }

        for (index, weapon) in self.weapons.iter().enumerate() {
            if weapon.id.is_empty() {
                return Err("weapon id must not be empty".to_string());
            }
            if weapon.model.is_empty() {
                return Err(format!("weapon '{}' must name a model", weapon.id));
            }
            if !finite3(weapon.model_rotation_deg) {
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
            if let Some(grip) = weapon.grip {
                grip.validate(&weapon.id)?;
            }
            if weapon.sockets.named().iter().any(|name| name.is_empty()) {
                return Err(format!("weapon '{}' has an empty socket name", weapon.id));
            }
            for pose in [&weapon.relaxed, &weapon.aim] {
                if !finite3(pose.offset) || !finite3(pose.rotation_deg) {
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
                return Err(format!("weapon '{}' has an invalid reload pose", weapon.id));
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
                return Err(format!("weapon '{}' has an invalid sway config", weapon.id));
            }
            let swap = &weapon.swap;
            if !swap.lower_secs.is_finite()
                || !swap.raise_secs.is_finite()
                || swap.lower_secs < 0.0
                || swap.raise_secs < 0.0
                || swap.total_secs() <= 0.0
            {
                return Err(format!(
                    "weapon '{}' has an invalid swap timeline",
                    weapon.id
                ));
            }
            for over in &weapon.socket_overrides {
                if over.socket.is_empty() {
                    return Err(format!(
                        "weapon '{}' has an empty socket override",
                        weapon.id
                    ));
                }
                if !finite3(over.offset) || !finite3(over.rotation_deg) {
                    return Err(format!(
                        "weapon '{}' socket override '{}' is non-finite",
                        weapon.id, over.socket
                    ));
                }
            }
            for other in &self.weapons[index + 1..] {
                if other.id == weapon.id {
                    return Err(format!("duplicate weapon id '{}'", weapon.id));
                }
            }
        }
        Ok(())
    }
}

fn finite3(value: [f32; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

impl RonConfig for WeaponCatalog {
    const PATH: &'static str = "config/player/weapons.ron";
}

#[cfg(test)]
mod tests;







