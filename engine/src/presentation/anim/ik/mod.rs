//! Arm IK that drives the character's hands onto the equipped weapon.
//!
//! The pipeline is: evaluate the animation, resolve the weapon's grip, place
//! the weapon, transform the grip targets into character space, solve both
//! arms with a two-bone IK, then apply the finger curls. Every write happens
//! after the animation evaluation so the animation and the IK never fight
//! over the same bone in the same frame.

mod solve;
mod target;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use crate::presentation::anim::catalog::AnimCatalog;
use crate::presentation::anim::grip::{FingerCurl, resolve_curl, resolve_grip};
use crate::presentation::anim::rig::{ArmChain, PlayerRig, compose_world};
use crate::presentation::anim::weapon::{
    PlayerWeaponAnchor, PlayerWeaponRuntime, WeaponMount, WeaponPoseState, socket_frames,
};
use crate::world::{Health, Survivor};

pub use solve::{ArmPose, ArmTarget, ChainState, ChainWrites, Solution, settle_delta, solve_arm, solve_chain, solve_two_bone};
pub use target::{WeaponGripTargets, build_targets};

/// Targets captured for the optional in-game IK gizmo.
#[cfg(feature = "dev-ik-gizmo")]
#[derive(Resource, Default)]
pub struct IkDebug {
    pub entries: Vec<IkDebugEntry>,
}

#[cfg(feature = "dev-ik-gizmo")]
#[derive(Debug, Clone, Copy)]
pub struct IkDebugEntry {
    pub root: Entity,
    pub primary: ArmTarget,
    pub primary_pole: Vec3,
    pub support: Option<(ArmTarget, Vec3)>,
    pub right: ArmChain,
    pub left: ArmChain,
}

/// Solves both arms onto the equipped weapon's grip.
///
/// The right hand takes the primary grip, the left hand the support grip. A
/// weapon without a support grip leaves the left arm on its animated pose.
pub fn solve_player_arms(
    catalog: Res<AnimCatalog>,
    rigs: Query<&PlayerRig>,
    mounts: Query<&WeaponMount>,
    anchors: Query<(&PlayerWeaponAnchor, &WeaponPoseState, &PlayerWeaponRuntime)>,
    survivors: Query<&Health, With<Survivor>>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
    #[cfg(feature = "dev-ik-gizmo")] mut debug: ResMut<IkDebug>,
) {
    if !catalog.ik.enabled {
        return;
    }

    #[cfg(feature = "dev-ik-gizmo")]
    debug.entries.clear();

    for mount in &mounts {
        let Ok(health) = survivors.get(mount.survivor) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        let Some(rig) = rigs.iter().find(|rig| rig.survivor == mount.survivor) else {
            continue;
        };
        if !rig.arms.right.is_complete() && !rig.arms.left.is_complete() {
            continue;
        }
        let Some((anchor, state, runtime)) = anchors
            .iter()
            .find(|(anchor, _, _)| anchor.survivor == mount.survivor)
        else {
            continue;
        };
        let weapon = &runtime.weapon;
        let progress = runtime.reload.progress();
        let hand_return = weapon.presentation.reload.hand_return;

        let (targets, grip) = {
            let read = transforms.p0();

            // The weapon's own space inside the character space.
            let Some(anchor_world) = compose_world(anchor.root, rig.root, &read, &parents) else {
                continue;
            };
            let weapon_frame = anchor_world.mul_transform(state.frame);

            let sockets = socket_frames(mount, &weapon_frame, &read, &parents, rig.root);
            let resolved = resolve_grip(weapon.presentation.grip.as_ref(), &sockets);

            let magazine = mount.sockets.magazine.and_then(|entity| {
                compose_world(entity, rig.root, &read, &parents).map(|frame| frame.translation)
            });

            let Some(targets) = build_targets(
                &resolved,
                &weapon_frame,
                magazine,
                progress,
                hand_return,
                wrist_offset(catalog.ik.right_hand_offset_deg),
                wrist_offset(catalog.ik.left_hand_offset_deg),
            ) else {
                continue;
            };
            (targets, resolved)
        };

        #[cfg(feature = "dev-ik-gizmo")]
        debug.entries.push(IkDebugEntry {
            root: rig.root,
            primary: targets.primary,
            primary_pole: targets.primary_pole,
            support: targets.support,
            right: rig.arms.right,
            left: rig.arms.left,
        });

        let writes = {
            let read = transforms.p0();
            let mut writes = Vec::with_capacity(2);
            if rig.arms.right.is_complete() {
                if let Some(chain) = solve_chain(
                    &rig.arms.right,
                    targets.primary,
                    targets.primary_pole,
                    &catalog.ik,
                    rig.root,
                    &read,
                    &parents,
                ) {
                    writes.push(chain);
                }
            }
            if let (Some(chain), Some((target, pole))) =
                (rig.arms.left.is_complete().then_some(&rig.arms.left), targets.support)
            {
                if let Some(chain) = solve_chain(
                    chain,
                    target,
                    pole,
                    &catalog.ik,
                    rig.root,
                    &read,
                    &parents,
                ) {
                    writes.push(chain);
                }
            }
            writes
        };

        let mut write = transforms.p1();
        for chain in writes {
            if let Some((entity, rotation)) = chain.shoulder
                && let Ok(mut transform) = write.get_mut(entity)
            {
                transform.rotation = rotation;
            }
            if let Ok(mut transform) = write.get_mut(chain.upper.0) {
                transform.rotation = chain.upper.1;
            }
            if let Ok(mut transform) = write.get_mut(chain.lower.0) {
                transform.rotation = chain.lower.1;
            }
            if let Some((entity, rotation)) = chain.hand
                && let Ok(mut transform) = write.get_mut(entity)
            {
                transform.rotation = rotation;
            }
        }

        apply_finger_curl(
            &rig.arms.right,
            &resolve_curl(&grip.primary, catalog.ik.right_finger_curl),
            &mut write,
        );
        if let Some(support) = grip.support {
            apply_finger_curl(
                &rig.arms.left,
                &resolve_curl(&support, catalog.ik.left_finger_curl),
                &mut write,
            );
        }
    }
}

/// Trims the solved hand rotation in the hand bone's own frame.
fn wrist_offset(degrees: [f32; 3]) -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        degrees[0].to_radians(),
        degrees[1].to_radians(),
        degrees[2].to_radians(),
    )
}

/// Applies a finger curl to the hand bones of one arm.
///
/// The player skeleton folds its fingers around the local X axis of each
/// bone, so the curl is a rotation about that axis.
fn apply_finger_curl(chain: &ArmChain, curl: &FingerCurl, write: &mut Query<&mut Transform, ()>) {
    for (bone_entity, degrees) in [
        (chain.middle_hand, curl.middle_hand_deg),
        (chain.fingers, curl.fingers_deg),
        (chain.thumb1, curl.thumb1_deg),
        (chain.thumb2, curl.thumb2_deg),
    ] {
        if degrees.abs() < f32::EPSILON {
            continue;
        }
        if let Some(entity) = bone_entity
            && let Ok(mut transform) = write.get_mut(entity)
        {
            let curl_rot = Quat::from_rotation_x(degrees.to_radians());
            transform.rotation = (transform.rotation * curl_rot).normalize();
        }
    }
}

/// Draws the resolved hand targets and arm chains for calibration.
#[cfg(feature = "dev-ik-gizmo")]
pub fn draw_ik_gizmo(
    debug: Res<IkDebug>,
    roots: Query<&GlobalTransform>,
    transforms: Query<&Transform>,
    parents: Query<&ChildOf>,
    mut gizmos: Gizmos,
) {
    for entry in &debug.entries {
        let Ok(root) = roots.get(entry.root) else {
            continue;
        };
        let root = root.compute_transform();

        let primary_position = root.transform_point(entry.primary.position);
        let primary_rotation = root.rotation * entry.primary.rotation;
        draw_frame(&mut gizmos, primary_position, primary_rotation, 0.06);
        draw_frame(
            &mut gizmos,
            primary_position,
            primary_rotation * entry.primary.wrist_offset,
            0.04,
        );
        gizmos.line(
            primary_position,
            primary_position
                + root.rotation * entry.primary_pole.normalize_or_zero() * 0.12,
            Color::srgb(1.0, 0.6, 0.1),
        );

        if let Some((support, pole)) = entry.support {
            let position = root.transform_point(support.position);
            let rotation = root.rotation * support.rotation;
            draw_frame(&mut gizmos, position, rotation, 0.06);
            draw_frame(
                &mut gizmos,
                position,
                rotation * support.wrist_offset,
                0.04,
            );
            gizmos.line(
                position,
                position + root.rotation * pole.normalize_or_zero() * 0.12,
                Color::srgb(1.0, 0.6, 0.1),
            );
        }

        draw_chain(
            &mut gizmos,
            &entry.right,
            entry.root,
            &transforms,
            &parents,
            &root,
            Color::srgb(0.2, 0.8, 1.0),
        );
        draw_chain(
            &mut gizmos,
            &entry.left,
            entry.root,
            &transforms,
            &parents,
            &root,
            Color::srgb(0.8, 0.3, 1.0),
        );
    }
}

#[cfg(feature = "dev-ik-gizmo")]
fn draw_frame(gizmos: &mut Gizmos, origin: Vec3, rotation: Quat, length: f32) {
    gizmos.line(
        origin,
        origin + rotation * Vec3::X * length,
        Color::srgb(1.0, 0.25, 0.25),
    );
    gizmos.line(
        origin,
        origin + rotation * Vec3::Y * length,
        Color::srgb(0.25, 1.0, 0.25),
    );
    gizmos.line(
        origin,
        origin + rotation * Vec3::Z * length,
        Color::srgb(0.25, 0.5, 1.0),
    );
}

#[cfg(feature = "dev-ik-gizmo")]
fn draw_chain(
    gizmos: &mut Gizmos,
    chain: &ArmChain,
    root: Entity,
    transforms: &Query<&Transform>,
    parents: &Query<&ChildOf>,
    root_transform: &Transform,
    color: Color,
) {
    let mut previous = None;
    for entity in [
        chain.shoulder,
        chain.upper_arm,
        chain.lower_arm,
        chain.hand,
    ]
    .into_iter()
    .flatten()
    {
        let Some(frame) = compose_world(entity, root, transforms, parents) else {
            continue;
        };
        let position = root_transform.transform_point(frame.translation);
        if let Some(previous) = previous {
            gizmos.line(previous, position, color);
        }
        previous = Some(position);
    }
}







