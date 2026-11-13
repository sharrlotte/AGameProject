use super::*;

impl Default for AnimCatalog {
    fn default() -> Self {
        Self {
            facing: FacingConfig::default(),
            locomotion: LocomotionConfig::default(),
            aim: AimConfig::default(),
            action_blend_secs: locomotion_blend(),
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
                locomotion("walk_back", "HumanArmature|Man_Walk"),
                locomotion("strafe_l", "HumanArmature|Man_StrafeL"),
                locomotion("strafe_r", "HumanArmature|Man_StrafeR"),
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







