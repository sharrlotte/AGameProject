use bevy::prelude::*;

use crate::presentation::anim::grip::{GripTarget, ResolvedGrip, WeaponTargets};

use super::solve::ArmTarget;

/// The grip of the equipped weapon, expressed in character space.
#[derive(Debug, Clone, Copy)]
pub struct WeaponGripTargets {
    pub primary: ArmTarget,
    /// World-space direction the primary elbow bends towards.
    pub primary_pole: Vec3,
    /// The support hand and its elbow pole, when the weapon is held with two hands.
    pub support: Option<(ArmTarget, Vec3)>,
}

/// Builds the arm targets of a grip in character space.
///
/// `weapon_frame` places the weapon's own space inside the character space and
/// `magazine` is the magazine's position in character space, used while
/// reloading. `primary_wrist` and `support_wrist` are the catalog fallbacks for
/// a hand that does not author its own wrist trim.
pub fn build_targets(
    grip: &ResolvedGrip,
    weapon_frame: &Transform,
    magazine: Option<Vec3>,
    reload_progress: Option<f32>,
    hand_return: f32,
    primary_wrist: Quat,
    support_wrist: Quat,
) -> Option<WeaponGripTargets> {
    let targets = WeaponTargets::from_grip(grip)?;
    let primary = arm_target(&targets.primary, weapon_frame, primary_wrist);
    let primary_pole = weapon_frame
        .rotation
        .mul_vec3(targets.primary.elbow.normalize_or_zero());

    let support = match targets.support {
        Some(support) => {
            let rest = arm_target(&support, weapon_frame, support_wrist);
            let target = match (magazine, reload_progress) {
                (Some(magazine), Some(progress)) => {
                    let t = if hand_return >= 1.0 {
                        1.0
                    } else {
                        ((progress - hand_return) / (1.0 - hand_return)).clamp(0.0, 1.0)
                    };
                    ArmTarget {
                        position: rest.position.lerp(magazine, t),
                        rotation: rest.rotation,
                        wrist_offset: rest.wrist_offset,
                    }
                }
                _ => rest,
            };
            Some((
                target,
                weapon_frame
                    .rotation
                    .mul_vec3(support.elbow.normalize_or_zero()),
            ))
        }
        None => None,
    };

    Some(WeaponGripTargets {
        primary,
        primary_pole,
        support,
    })
}

fn arm_target(target: &GripTarget, weapon_frame: &Transform, wrist: Quat) -> ArmTarget {
    ArmTarget {
        position: weapon_frame.transform_point(target.position),
        rotation: (weapon_frame.rotation * target.rotation()).normalize(),
        wrist_offset: target.wrist(wrist),
    }
}







