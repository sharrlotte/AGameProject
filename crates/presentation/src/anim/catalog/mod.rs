use bevy::prelude::*;
use data::RonConfig;
use serde::Deserialize;

/// Which animation layer a clip belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimLayer {
    Locomotion,
    Action,
    Additive,
    Override,
}

/// One clip role in the animation catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct ClipDef {
    /// Stable role name used by gameplay code, for example `walk_fwd`.
    pub role: String,
    /// glTF animation label, for example `HumanArmature|Man_Walk`.
    pub clip: String,
    pub layer: AnimLayer,
    #[serde(default)]
    pub looped: bool,
    #[serde(default = "one")]
    pub speed: f32,
}

fn one() -> f32 {
    1.0
}

/// A mask group is an explicit bone list because `Hips` is the ancestor of the spine.
#[derive(Debug, Clone, Deserialize)]
pub struct MaskGroupDef {
    pub name: String,
    pub bones: Vec<String>,
}

/// One directional locomotion pose.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Anchor {
    pub role: String,
    /// Character-local angle: `0` forward, `+90` right, `180` backward, `-90` left.
    pub angle_deg: f32,
    /// Optional sprint variant clip role (e.g. `sprint_fwd`). When set and the simulation
    /// reports sprinting, the locomotion layer crossfades from `role` to this clip.
    #[serde(default)]
    pub sprint_role: Option<String>,
}

impl Anchor {
    pub fn angle_rad(&self) -> f32 {
        self.angle_deg.to_radians()
    }
}

fn yes() -> bool {
    true
}

fn facing_blend() -> f32 {
    0.12
}

/// Aim-facing yaw smoothing.
#[derive(Debug, Clone, Deserialize)]
pub struct FacingConfig {
    #[serde(default = "facing_blend")]
    pub blend_secs: f32,
}

impl Default for FacingConfig {
    fn default() -> Self {
        Self {
            blend_secs: facing_blend(),
        }
    }
}

fn idle_role() -> String {
    "idle".to_string()
}

fn move_threshold() -> f32 {
    0.25
}

fn walk_speed_ref() -> f32 {
    7.5
}

fn gait_blend() -> f32 {
    0.15
}

fn sprint_speed_ref() -> f32 {
    8.0
}

fn sprint_playback() -> f32 {
    1.35
}

fn speed_min() -> f32 {
    0.6
}

fn speed_max() -> f32 {
    1.6
}

fn locomotion_blend() -> f32 {
    0.15
}

fn cardinal_anchors() -> Vec<Anchor> {
    vec![
        Anchor {
            role: "walk_fwd".to_string(),
            angle_deg: 0.0,
            sprint_role: None,
        },
        Anchor {
            role: "strafe_r".to_string(),
            angle_deg: 90.0,
            sprint_role: None,
        },
        Anchor {
            role: "walk_back".to_string(),
            angle_deg: 180.0,
            sprint_role: None,
        },
        Anchor {
            role: "strafe_l".to_string(),
            angle_deg: -90.0,
            sprint_role: None,
        },
    ]
}

/// Anchor-based directional locomotion and playback settings.
#[derive(Debug, Clone, Deserialize)]
pub struct LocomotionConfig {
    #[serde(default = "idle_role")]
    pub idle_role: String,
    #[serde(default = "move_threshold")]
    pub move_speed_threshold: f32,
    #[serde(default = "yes")]
    pub phase_sync: bool,
    #[serde(default = "walk_speed_ref")]
    pub walk_speed_ref: f32,
    #[serde(default = "sprint_playback")]
    pub sprint_playback_speed: f32,
    #[serde(default = "gait_blend")]
    pub gait_blend_secs: f32,
    #[serde(default = "sprint_speed_ref")]
    pub sprint_speed_ref: f32,
    #[serde(default = "speed_min")]
    pub speed_scale_min: f32,
    #[serde(default = "speed_max")]
    pub speed_scale_max: f32,
    #[serde(default = "locomotion_blend")]
    pub blend_secs: f32,
    #[serde(default = "cardinal_anchors")]
    pub anchors: Vec<Anchor>,
}

impl Default for LocomotionConfig {
    fn default() -> Self {
        Self {
            idle_role: idle_role(),
            move_speed_threshold: move_threshold(),
            phase_sync: true,
            walk_speed_ref: walk_speed_ref(),
            sprint_playback_speed: sprint_playback(),
            gait_blend_secs: gait_blend(),
            sprint_speed_ref: sprint_speed_ref(),
            speed_scale_min: speed_min(),
            speed_scale_max: speed_max(),
            blend_secs: locomotion_blend(),
            anchors: cardinal_anchors(),
        }
    }
}

fn aim_blend() -> f32 {
    0.12
}

fn aim_height() -> f32 {
    1.05
}

/// Weapon aim pose timing.
#[derive(Debug, Clone, Deserialize)]
pub struct AimConfig {
    #[serde(default = "aim_blend")]
    pub blend_secs: f32,
    #[serde(default = "aim_height")]
    pub aim_height: f32,
}

impl Default for AimConfig {
    fn default() -> Self {
        Self {
            blend_secs: aim_blend(),
            aim_height: aim_height(),
        }
    }
}

fn down_back() -> [f32; 3] {
    [0.0, -0.3, -1.0]
}

fn right_chain() -> Vec<String> {
    vec![
        "UpperArm.R".to_string(),
        "LowerArm.R".to_string(),
        "Palm.R".to_string(),
    ]
}

fn left_chain() -> Vec<String> {
    vec![
        "UpperArm.L".to_string(),
        "LowerArm.L".to_string(),
        "Palm.L".to_string(),
    ]
}

fn right_shoulder() -> String {
    "Shoulder.R".to_string()
}

fn left_shoulder() -> String {
    "Shoulder.L".to_string()
}

fn shoulder_settle() -> f32 {
    0.35
}

fn settle_limit() -> f32 {
    25.0
}

/// Per-finger curl angles (degrees) applied after arm IK.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
pub struct FingerCurl {
    #[serde(default)]
    pub middle_hand_deg: [f32; 3],
    #[serde(default)]
    pub fingers_deg: [f32; 3],
    #[serde(default)]
    pub thumb1_deg: [f32; 3],
    #[serde(default)]
    pub thumb2_deg: [f32; 3],
}

/// Dual-arm hand IK settings.
#[derive(Debug, Clone, Deserialize)]
pub struct IkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "right_chain")]
    pub right_chain: Vec<String>,
    #[serde(default = "left_chain")]
    pub left_chain: Vec<String>,
    #[serde(default = "right_shoulder")]
    pub right_shoulder: String,
    #[serde(default = "left_shoulder")]
    pub left_shoulder: String,
    #[serde(default = "down_back")]
    pub right_pole: [f32; 3],
    #[serde(default = "down_back")]
    pub left_pole: [f32; 3],
    #[serde(default)]
    pub hand_align: bool,
    #[serde(default)]
    pub right_hand_offset_deg: [f32; 3],
    #[serde(default)]
    pub left_hand_offset_deg: [f32; 3],
    #[serde(default)]
    pub hand_offset_deg: [f32; 3],
    #[serde(default = "shoulder_settle")]
    pub shoulder_settle: f32,
    #[serde(default = "settle_limit")]
    pub settle_limit_deg: f32,
    #[serde(default)]
    pub right_finger_curl: FingerCurl,
    #[serde(default)]
    pub left_finger_curl: FingerCurl,
}

impl Default for IkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            right_chain: right_chain(),
            left_chain: left_chain(),
            right_shoulder: right_shoulder(),
            left_shoulder: left_shoulder(),
            right_pole: down_back(),
            left_pole: down_back(),
            hand_align: true,
            right_hand_offset_deg: [0.0; 3],
            left_hand_offset_deg: [0.0; 3],
            hand_offset_deg: [0.0; 3],
            shoulder_settle: shoulder_settle(),
            settle_limit_deg: settle_limit(),
            right_finger_curl: FingerCurl::default(),
            left_finger_curl: FingerCurl::default(),
        }
    }
}

/// Hot-reloadable animation configuration.
#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct AnimCatalog {
    #[serde(default)]
    pub facing: FacingConfig,
    #[serde(default)]
    pub locomotion: LocomotionConfig,
    #[serde(default)]
    pub aim: AimConfig,
    #[serde(default = "locomotion_blend")]
    pub action_blend_secs: f32,
    pub mask_groups: Vec<MaskGroupDef>,
    pub clips: Vec<ClipDef>,
    #[serde(default)]
    pub ik: IkConfig,
}

impl AnimCatalog {
    /// Roles that must exist in the catalog. The GLB may lag behind while clips are authored.
    pub const REQUIRED_ROLES: &'static [&'static str] = &[
        "idle",
        "walk_fwd",
        "walk_back",
        "strafe_l",
        "strafe_r",
        "death",
    ];

    pub fn clip(&self, role: &str) -> Option<&ClipDef> {
        self.clips.iter().find(|clip| clip.role == role)
    }

    pub fn mask_group(&self, name: &str) -> Option<&MaskGroupDef> {
        self.mask_groups.iter().find(|group| group.name == name)
    }

    pub fn required_ik_chain_len(&self) -> usize {
        3
    }

    /// Validates anchors and playback settings before the graph is built.
    pub fn validate(&self) -> Result<(), String> {
        for role in Self::REQUIRED_ROLES {
            if self.clip(role).is_none() {
                return Err(format!("animation catalog is missing clip role '{role}'"));
            }
        }
        for name in ["lower", "upper"] {
            let Some(group) = self.mask_group(name) else {
                return Err(format!("animation catalog is missing mask group '{name}'"));
            };
            if group.bones.is_empty() {
                return Err(format!("mask group '{name}' has no bones"));
            }
        }

        let loco = &self.locomotion;
        if loco.anchors.is_empty() {
            return Err("locomotion must define at least one anchor".to_string());
        }
        if loco.idle_role.is_empty() || self.clip(&loco.idle_role).is_none() {
            return Err("locomotion idle_role must name an existing clip role".to_string());
        }
        if !loco.move_speed_threshold.is_finite() || loco.move_speed_threshold < 0.0 {
            return Err("locomotion.move_speed_threshold must be non-negative".to_string());
        }
        if !loco.phase_sync {
            return Err("locomotion.phase_sync must stay enabled".to_string());
        }
        if !loco.walk_speed_ref.is_finite() || loco.walk_speed_ref <= 0.0 {
            return Err("locomotion.walk_speed_ref must be greater than zero".to_string());
        }
        if !loco.sprint_playback_speed.is_finite() || loco.sprint_playback_speed <= 0.0 {
            return Err("locomotion.sprint_playback_speed must be greater than zero".to_string());
        }
        if !loco.gait_blend_secs.is_finite() || loco.gait_blend_secs < 0.0 {
            return Err("locomotion.gait_blend_secs must be non-negative".to_string());
        }
        if !loco.sprint_speed_ref.is_finite() || loco.sprint_speed_ref <= 0.0 {
            return Err("locomotion.sprint_speed_ref must be greater than zero".to_string());
        }
        if loco.speed_scale_min <= 0.0 || loco.speed_scale_max < loco.speed_scale_min {
            return Err("locomotion speed scale range is invalid".to_string());
        }
        if !loco.blend_secs.is_finite() || loco.blend_secs < 0.0 {
            return Err("locomotion.blend_secs must be non-negative".to_string());
        }

        for anchor in &loco.anchors {
            if anchor.role.is_empty() || self.clip(&anchor.role).is_none() {
                return Err(format!(
                    "locomotion anchor role '{}' must name an existing clip role",
                    anchor.role
                ));
            }
            if let Some(sprint_role) = &anchor.sprint_role
                && (sprint_role.is_empty() || self.clip(sprint_role).is_none())
            {
                return Err(format!(
                    "locomotion anchor sprint_role '{}' must name an existing clip role",
                    sprint_role
                ));
            }
            if !anchor.angle_deg.is_finite() || !(-180.0..=180.0).contains(&anchor.angle_deg) {
                return Err(format!(
                    "locomotion anchor '{}' angle must be within [-180, 180]",
                    anchor.role
                ));
            }
        }
        for (index, anchor) in loco.anchors.iter().enumerate() {
            for other in &loco.anchors[index + 1..] {
                if (anchor.angle_deg - other.angle_deg).abs() <= 1e-3 {
                    return Err(format!(
                        "locomotion anchors '{}' and '{}' share angle {}",
                        anchor.role, other.role, anchor.angle_deg
                    ));
                }
            }
        }

        if !self.facing.blend_secs.is_finite() || self.facing.blend_secs < 0.0 {
            return Err("facing.blend_secs must be non-negative".to_string());
        }
        if !self.aim.blend_secs.is_finite() || self.aim.blend_secs < 0.0 {
            return Err("aim.blend_secs must be non-negative".to_string());
        }
        if !self.aim.aim_height.is_finite() || self.aim.aim_height <= 0.0 {
            return Err("aim.aim_height must be greater than zero".to_string());
        }
        if !self.action_blend_secs.is_finite() || self.action_blend_secs < 0.0 {
            return Err("action_blend_secs must be non-negative".to_string());
        }

        if self.ik.enabled {
            for (label, chain) in [
                ("ik.right_chain", &self.ik.right_chain),
                ("ik.left_chain", &self.ik.left_chain),
            ] {
                if chain.len() != self.required_ik_chain_len() {
                    return Err(format!(
                        "{label} must contain {} bones, got {}",
                        self.required_ik_chain_len(),
                        chain.len()
                    ));
                }
                if chain.iter().any(String::is_empty) {
                    return Err(format!("{label} contains an empty bone name"));
                }
            }
            if self.ik.right_shoulder.is_empty() || self.ik.left_shoulder.is_empty() {
                return Err("ik shoulder bone names must not be empty".to_string());
            }
            if !self.ik.shoulder_settle.is_finite()
                || !(0.0..=1.0).contains(&self.ik.shoulder_settle)
            {
                return Err("ik.shoulder_settle must be within [0, 1]".to_string());
            }
            if !self.ik.settle_limit_deg.is_finite()
                || self.ik.settle_limit_deg < 0.0
                || self.ik.settle_limit_deg > 180.0
            {
                return Err("ik.settle_limit_deg must be within [0, 180]".to_string());
            }
        }
        Ok(())
    }
}

mod preset;

impl RonConfig for AnimCatalog {
    const PATH: &'static str = "config/player/animation.ron";
}

#[cfg(test)]
mod tests;
