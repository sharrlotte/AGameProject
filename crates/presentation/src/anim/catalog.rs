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

/// Runtime hand IK settings.
#[derive(Debug, Clone, Deserialize)]
pub struct IkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub chain: Vec<String>,
    #[serde(default)]
    pub target_bone: String,
    #[serde(default = "down_back")]
    pub pole: [f32; 3],
    #[serde(default)]
    pub hand_align: bool,
    #[serde(default)]
    pub hand_offset_deg: [f32; 3],
}

fn down_back() -> [f32; 3] {
    [0.0, -0.3, -1.0]
}

impl Default for IkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chain: vec![
                "UpperArm.L".to_string(),
                "LowerArm.L".to_string(),
                "Palm.L".to_string(),
            ],
            target_bone: "Foregrip".to_string(),
            pole: down_back(),
            hand_align: true,
            hand_offset_deg: [0.0, 0.0, 0.0],
        }
    }
}

fn yes() -> bool {
    true
}

fn default_max_twist_deg() -> f32 {
    45.0
}

fn default_hysteresis_deg() -> f32 {
    5.0
}

fn default_idle_turn_stop_deg() -> f32 {
    5.0
}

fn default_twist_blend_secs() -> f32 {
    0.10
}

fn default_twist_bones() -> Vec<TwistBone> {
    vec![
        TwistBone("Hips".to_string(), 0.25),
        TwistBone("Abdomen".to_string(), 0.35),
        TwistBone("Torso".to_string(), 0.40),
    ]
}

/// Bone name and normalized share of procedural upper-body yaw twist.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TwistBone(pub String, pub f32);

impl TwistBone {
    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn weight(&self) -> f32 {
        self.1
    }
}

/// Upper-body spine twist and lower-body cone thresholds.
#[derive(Debug, Clone, Deserialize)]
pub struct TwistConfig {
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default = "default_max_twist_deg")]
    pub max_twist_deg: f32,
    #[serde(default = "default_hysteresis_deg")]
    pub hysteresis_deg: f32,
    #[serde(default = "default_idle_turn_stop_deg")]
    pub idle_turn_stop_deg: f32,
    #[serde(default = "default_twist_blend_secs")]
    pub twist_blend_secs: f32,
    #[serde(default = "default_twist_bones")]
    pub bones: Vec<TwistBone>,
}

impl Default for TwistConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_twist_deg: default_max_twist_deg(),
            hysteresis_deg: default_hysteresis_deg(),
            idle_turn_stop_deg: default_idle_turn_stop_deg(),
            twist_blend_secs: default_twist_blend_secs(),
            bones: default_twist_bones(),
        }
    }
}

/// Hot-reloadable animation configuration.
#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct AnimCatalog {
    pub walk_speed_ref: f32,
    pub sprint_playback_speed: f32,
    pub speed_scale_min: f32,
    pub speed_scale_max: f32,
    pub locomotion_blend_secs: f32,
    pub action_blend_secs: f32,
    pub facing_blend_secs: f32,
    pub mask_groups: Vec<MaskGroupDef>,
    pub clips: Vec<ClipDef>,
    pub ik: IkConfig,
    #[serde(default)]
    pub twist: TwistConfig,
}

impl AnimCatalog {
    /// Roles that must exist in the catalog. The GLB may lag behind while clips are authored.
    pub const REQUIRED_ROLES: &'static [&'static str] = &[
        "idle", "walk_fwd", "sprint", "strafe_l", "strafe_r", "aim", "reload", "death",
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
        if self.ik.enabled {
            if self.ik.chain.len() != self.required_ik_chain_len() {
                return Err(format!(
                    "IK chain must contain {} bones, got {}",
                    self.required_ik_chain_len(),
                    self.ik.chain.len()
                ));
            }
            if self.ik.target_bone.is_empty() {
                return Err("IK target bone is empty".to_string());
            }
        }
        if self.walk_speed_ref <= 0.0 {
            return Err("walk_speed_ref must be greater than zero".to_string());
        }
        if self.twist.enabled {
            if !self.twist.max_twist_deg.is_finite()
                || self.twist.max_twist_deg <= 0.0
                || self.twist.max_twist_deg > 90.0
            {
                return Err("twist.max_twist_deg must be within (0, 90]".to_string());
            }
            if !self.twist.hysteresis_deg.is_finite()
                || self.twist.hysteresis_deg < 0.0
                || self.twist.hysteresis_deg >= self.twist.max_twist_deg
            {
                return Err("twist.hysteresis_deg must be in [0, max_twist_deg)".to_string());
            }
            if !self.twist.idle_turn_stop_deg.is_finite()
                || self.twist.idle_turn_stop_deg < 0.0
                || self.twist.idle_turn_stop_deg > self.twist.max_twist_deg
            {
                return Err("twist.idle_turn_stop_deg must be in [0, max_twist_deg]".to_string());
            }
            if !self.twist.twist_blend_secs.is_finite() || self.twist.twist_blend_secs < 0.0 {
                return Err("twist.twist_blend_secs must be non-negative".to_string());
            }
            if self.twist.bones.is_empty() {
                return Err("twist.bones must not be empty when twist is enabled".to_string());
            }
            let mut total_weight = 0.0;
            for entry in &self.twist.bones {
                if entry.name().is_empty() {
                    return Err("twist bone name must not be empty".to_string());
                }
                if !entry.weight().is_finite() || entry.weight() < 0.0 {
                    return Err(format!("twist bone '{}' has invalid weight", entry.name()));
                }
                total_weight += entry.weight();
            }
            if total_weight <= 1e-4 {
                return Err("twist.bones total weight must be greater than zero".to_string());
            }
        }
        Ok(())
    }
}

impl Default for AnimCatalog {
    fn default() -> Self {
        Self {
            walk_speed_ref: 7.5,
            sprint_playback_speed: 1.35,
            speed_scale_min: 0.6,
            speed_scale_max: 1.6,
            locomotion_blend_secs: 0.15,
            action_blend_secs: 0.12,
            facing_blend_secs: 0.12,
            mask_groups: vec![
                MaskGroupDef {
                    name: "lower".to_string(),
                    bones: vec![
                        "Body".to_string(),
                        "Bone".to_string(),
                        "Hips".to_string(),
                        "UpperLeg.L".to_string(),
                        "LowerLeg.L".to_string(),
                        "Foot.L".to_string(),
                        "UpperLeg.R".to_string(),
                        "LowerLeg.R".to_string(),
                        "Foot.R".to_string(),
                    ],
                },
                MaskGroupDef {
                    name: "upper".to_string(),
                    bones: vec![
                        "Abdomen".to_string(),
                        "Torso".to_string(),
                        "Neck".to_string(),
                        "Head".to_string(),
                        "Shoulder.L".to_string(),
                        "UpperArm.L".to_string(),
                        "LowerArm.L".to_string(),
                        "Palm.L".to_string(),
                        "MiddleHand.L".to_string(),
                        "Fingers.L".to_string(),
                        "Thumb1.L".to_string(),
                        "Thumb2.L".to_string(),
                        "Shoulder.R".to_string(),
                        "UpperArm.R".to_string(),
                        "LowerArm.R".to_string(),
                        "Palm.R".to_string(),
                        "MiddleHand.R".to_string(),
                        "Fingers.R".to_string(),
                        "Thumb1.R".to_string(),
                        "Thumb2.R".to_string(),
                    ],
                },
            ],
            clips: vec![
                locomotion("idle", "HumanArmature|Man_Idle"),
                locomotion("walk_fwd", "HumanArmature|Man_Walk"),
                locomotion("sprint", "HumanArmature|Man_Run"),
                locomotion("strafe_l", "HumanArmature|Man_StrafeL"),
                locomotion("strafe_r", "HumanArmature|Man_StrafeR"),
                action("aim", "HumanArmature|Man_Aim", true),
                action("reload", "HumanArmature|Man_Reload", false),
                ClipDef {
                    role: "death".to_string(),
                    clip: "HumanArmature|Man_Death".to_string(),
                    layer: AnimLayer::Override,
                    looped: false,
                    speed: 1.0,
                },
            ],
            ik: IkConfig::default(),
            twist: TwistConfig::default(),
        }
    }
}

fn locomotion(role: &str, clip: &str) -> ClipDef {
    ClipDef {
        role: role.to_string(),
        clip: clip.to_string(),
        layer: AnimLayer::Locomotion,
        looped: true,
        speed: 1.0,
    }
}

fn action(role: &str, clip: &str, looped: bool) -> ClipDef {
    ClipDef {
        role: role.to_string(),
        clip: clip.to_string(),
        layer: AnimLayer::Action,
        looped,
        speed: 1.0,
    }
}

impl RonConfig for AnimCatalog {
    const PATH: &'static str = "config/player/animation.ron";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_ron_parses_and_is_valid() {
        let catalog: AnimCatalog = data::assert_ron_config_parses!(AnimCatalog);
        catalog.validate().expect("animation catalog is valid");

        for role in AnimCatalog::REQUIRED_ROLES {
            assert!(catalog.clip(role).is_some(), "missing clip role {role}");
        }
        assert!(catalog.mask_group("lower").is_some());
        assert!(catalog.mask_group("upper").is_some());
        assert_eq!(catalog.ik.chain.len(), 3);
        assert!(catalog.twist.enabled);
        assert!((catalog.twist.max_twist_deg - 45.0).abs() < 1e-4);
        assert_eq!(catalog.twist.bones.len(), 3);
    }

    #[test]
    fn twist_validation_rejects_invalid_thresholds() {
        let mut catalog = AnimCatalog::default();
        catalog.twist.max_twist_deg = 120.0;
        assert!(catalog.validate().is_err());

        let mut catalog = AnimCatalog::default();
        catalog.twist.hysteresis_deg = 50.0;
        assert!(catalog.validate().is_err());

        let mut catalog = AnimCatalog::default();
        catalog.twist.bones.clear();
        assert!(catalog.validate().is_err());
    }
}
