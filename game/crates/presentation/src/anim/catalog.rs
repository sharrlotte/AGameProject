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
    const PATH: &'static str = "config/animation.ron";
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
    }
}
