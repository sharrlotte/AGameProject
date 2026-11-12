use bevy::prelude::*;

use crate::anim::catalog::{AnimCatalog, FingerCurl, IkConfig};
use crate::anim::rig::{ArmChain, PlayerRig, compose_world, parent_of};
use crate::anim::weapon::{WeaponCatalog, WeaponMount};
use combat::WeaponInventory;
use world::{Health, Survivor};

const EPSILON: f32 = 1e-4;

/// Result of the analytic two-bone solve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub elbow: Vec3,
    pub hand: Vec3,
}

/// Where a hand should sit and which way it should face.
#[derive(Debug, Clone, Copy)]
pub struct ArmTarget {
    pub position: Vec3,
    pub rotation: Quat,
}

/// Solved world-space rotations for one arm.
#[derive(Debug, Clone, Copy)]
pub struct ArmPose {
    pub upper_world: Quat,
    pub lower_world: Quat,
    pub hand_world: Quat,
}

/// Current frame of a two-bone chain before targeting.
#[derive(Debug, Clone, Copy)]
pub struct ChainState {
    pub upper_pos: Vec3,
    pub mid_pos: Vec3,
    pub hand_pos: Vec3,
    pub upper_rot: Quat,
    pub mid_rot: Quat,
    pub hand_rot: Quat,
}

/// Solves elbow and hand positions for a two-bone chain.
///
/// `root` is the shoulder, `mid` the elbow, `end` the hand. `pole_hint` points
/// roughly away from the desired bend direction. Unreachable targets are
/// clamped to the chain's reach.
pub fn solve_two_bone(
    root: Vec3,
    mid: Vec3,
    end: Vec3,
    target: Vec3,
    pole_hint: Vec3,
) -> Option<Solution> {
    let upper_len = (mid - root).length();
    let lower_len = (end - mid).length();
    if !upper_len.is_finite()
        || !lower_len.is_finite()
        || upper_len <= EPSILON
        || lower_len <= EPSILON
    {
        return None;
    }

    let to_target = target - root;
    let raw_distance = to_target.length();
    if !raw_distance.is_finite() || raw_distance <= EPSILON {
        return None;
    }
    let direction = to_target / raw_distance;

    let max_reach = upper_len + lower_len - EPSILON;
    let min_reach = (upper_len - lower_len).abs() + EPSILON;
    let low = min_reach.min(max_reach);
    let high = max_reach.max(min_reach);
    let distance = raw_distance.clamp(low, high);
    let hand = root + direction * distance;

    let mut pole = pole_hint - direction * pole_hint.dot(direction);
    if !pole.is_finite() || pole.length_squared() <= EPSILON {
        pole = direction.any_orthonormal_vector();
    }
    let pole = pole.normalize();

    let along =
        (upper_len * upper_len - lower_len * lower_len + distance * distance) / (2.0 * distance);
    let height_sq = (upper_len * upper_len - along * along).max(0.0);
    let elbow = root + direction * along + pole * height_sq.sqrt();

    if !elbow.is_finite() || !hand.is_finite() {
        return None;
    }
    Some(Solution { elbow, hand })
}

/// Rotates the current chain to reach an arbitrary target frame.
pub fn solve_arm(
    chain: ChainState,
    target: ArmTarget,
    pole_hint: Vec3,
    align: bool,
    hand_offset: Quat,
) -> Option<ArmPose> {
    let solution = solve_two_bone(
        chain.upper_pos,
        chain.mid_pos,
        chain.hand_pos,
        target.position,
        pole_hint,
    )?;

    let current_upper = (chain.mid_pos - chain.upper_pos).normalize_or_zero();
    let desired_upper = (solution.elbow - chain.upper_pos).normalize_or_zero();
    if current_upper == Vec3::ZERO || desired_upper == Vec3::ZERO {
        return None;
    }
    let upper_world =
        (Quat::from_rotation_arc(current_upper, desired_upper) * chain.upper_rot).normalize();

    let current_lower = (chain.hand_pos - chain.mid_pos).normalize_or_zero();
    let desired_lower = (solution.hand - solution.elbow).normalize_or_zero();
    if current_lower == Vec3::ZERO || desired_lower == Vec3::ZERO {
        return None;
    }
    let lower_world =
        (Quat::from_rotation_arc(current_lower, desired_lower) * chain.mid_rot).normalize();

    let hand_world = if align {
        (target.rotation * hand_offset).normalize()
    } else {
        chain.hand_rot
    };

    if !upper_world.is_finite() || !lower_world.is_finite() || !hand_world.is_finite() {
        return None;
    }
    Some(ArmPose {
        upper_world,
        lower_world,
        hand_world,
    })
}

/// Yaw/pitch-style shoulder follow, scaled by weight and clamped to a limit.
pub fn settle_delta(current_dir: Vec3, desired_dir: Vec3, weight: f32, limit_rad: f32) -> Quat {
    let from = current_dir.normalize_or_zero();
    let to = desired_dir.normalize_or_zero();
    if from == Vec3::ZERO || to == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    let delta = Quat::from_rotation_arc(from, to);
    if !delta.is_finite() {
        return Quat::IDENTITY;
    }
    let (axis, angle) = delta.to_axis_angle();
    if axis.length_squared() <= EPSILON || angle <= EPSILON {
        return Quat::IDENTITY;
    }
    let scaled = (angle * weight.clamp(0.0, 1.0)).min(limit_rad.max(0.0));
    Quat::from_axis_angle(axis, scaled).normalize()
}

/// Local rotations to write back for one solved arm.
struct ChainWrites {
    shoulder: Option<(Entity, Quat)>,
    upper: (Entity, Quat),
    lower: (Entity, Quat),
    hand: Option<(Entity, Quat)>,
}

fn solve_chain(
    chain: &ArmChain,
    target: ArmTarget,
    pole: [f32; 3],
    hand_offset: Quat,
    ik: &IkConfig,
    root: Entity,
    transforms: &Query<&Transform>,
    parents: &Query<&ChildOf>,
) -> Option<ChainWrites> {
    let (Some(upper_entity), Some(mid_entity), Some(hand_entity)) =
        (chain.upper_arm, chain.lower_arm, chain.hand)
    else {
        return None;
    };

    let upper_world = compose_world(upper_entity, root, transforms, parents)?;
    let mid_world = compose_world(mid_entity, root, transforms, parents)?;
    let hand_world = compose_world(hand_entity, root, transforms, parents)?;

    let mut shoulder_write = None;
    let mut upper_pos = upper_world.translation;
    let mut mid_pos = mid_world.translation;
    let mut hand_pos = hand_world.translation;
    let mut upper_rot = upper_world.rotation;
    let mut mid_rot = mid_world.rotation;

    let upper_parent_rot = if let Some(shoulder_entity) = chain.shoulder {
        let shoulder_world = compose_world(shoulder_entity, root, transforms, parents)?;
        let parent_entity = parent_of(shoulder_entity, parents)?;
        let shoulder_parent = compose_world(parent_entity, root, transforms, parents)?;

        let current_dir = (mid_pos - upper_pos).normalize_or_zero();
        let desired_dir = (target.position - upper_pos).normalize_or_zero();
        let delta = settle_delta(
            current_dir,
            desired_dir,
            ik.shoulder_settle,
            ik.settle_limit_deg.to_radians(),
        );

        if delta != Quat::IDENTITY {
            let shoulder_world_rot = (delta * shoulder_world.rotation).normalize();
            let shoulder_local =
                (shoulder_parent.rotation.inverse() * shoulder_world_rot).normalize();
            if !shoulder_local.is_finite() {
                return None;
            }
            shoulder_write = Some((shoulder_entity, shoulder_local));

            upper_pos = shoulder_world.translation
                + delta * (upper_world.translation - shoulder_world.translation);
            mid_pos = upper_pos + delta * (mid_world.translation - upper_world.translation);
            hand_pos = upper_pos + delta * (hand_world.translation - upper_world.translation);
            upper_rot = (delta * upper_world.rotation).normalize();
            mid_rot = (delta * mid_world.rotation).normalize();

            shoulder_world_rot
        } else {
            shoulder_world.rotation
        }
    } else {
        let parent_entity = parent_of(upper_entity, parents)?;
        compose_world(parent_entity, root, transforms, parents)?.rotation
    };

    let pole_hint = upper_parent_rot * Vec3::from_array(pole);
    let pose = solve_arm(
        ChainState {
            upper_pos,
            mid_pos,
            hand_pos,
            upper_rot,
            mid_rot,
            hand_rot: hand_world.rotation,
        },
        target,
        pole_hint,
        ik.hand_align,
        hand_offset,
    )?;

    let upper_local = (upper_parent_rot.inverse() * pose.upper_world).normalize();
    let lower_local = (pose.upper_world.inverse() * pose.lower_world).normalize();
    let hand_local = (pose.lower_world.inverse() * pose.hand_world).normalize();
    if !upper_local.is_finite() || !lower_local.is_finite() || !hand_local.is_finite() {
        return None;
    }

    Some(ChainWrites {
        shoulder: shoulder_write,
        upper: (upper_entity, upper_local),
        lower: (mid_entity, lower_local),
        hand: Some((hand_entity, hand_local)),
    })
}

/// Solves both arms to the weapon sockets after animation evaluation.
///
/// The right hand targets `Grip`; the left targets `Foregrip`, or `Magazine`
/// while reloading and blends back during the final part of the reload.
#[allow(clippy::too_many_arguments)]
pub fn solve_player_arms(
    catalog: Res<AnimCatalog>,
    weapons: Res<WeaponCatalog>,
    rigs: Query<&PlayerRig>,
    mounts: Query<&WeaponMount>,
    survivors: Query<&Health, With<Survivor>>,
    inventories: Query<&WeaponInventory>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
) {
    if !catalog.ik.enabled {
        return;
    }
    for mount in &mounts {
        info!("IK: mount profile={} survivor={:?}", mount.profile, mount.survivor);
    }

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

        let progress = inventories
            .get(mount.survivor)
            .ok()
            .and_then(WeaponInventory::reload_progress);
        let hand_return = weapons
            .weapon(&mount.profile)
            .map(|profile| profile.reload.hand_return)
            .unwrap_or(0.7);

        let (right_target, left_target) = {
            let read = transforms.p0();
            let Some(grip) = compose_world(mount.sockets.grip, rig.root, &read, &parents) else {
                continue;
            };
            let Some(foregrip) = compose_world(mount.sockets.foregrip, rig.root, &read, &parents)
            else {
                continue;
            };

            let left = if let (Some(progress), Some(magazine)) = (progress, mount.sockets.magazine)
            {
                let magazine_world =
                    compose_world(magazine, rig.root, &read, &parents).unwrap_or(foregrip);
                let t = if hand_return >= 1.0 {
                    1.0
                } else {
                    ((progress - hand_return) / (1.0 - hand_return)).clamp(0.0, 1.0)
                };
                lerp_transform(magazine_world, foregrip, t)
            } else {
                foregrip
            };

            (
                ArmTarget {
                    position: grip.translation,
                    rotation: grip.rotation,
                },
                ArmTarget {
                    position: left.translation,
                    rotation: left.rotation,
                },
            )
        };

        let writes = {
            let read = transforms.p0();
            [
                solve_chain(
                    &rig.arms.right,
                    right_target,
                    catalog.ik.right_pole,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        catalog.ik.right_hand_offset_deg[0].to_radians(),
                        catalog.ik.right_hand_offset_deg[1].to_radians(),
                        catalog.ik.right_hand_offset_deg[2].to_radians(),
                    ),
                    &catalog.ik,
                    rig.root,
                    &read,
                    &parents,
                ),
                solve_chain(
                    &rig.arms.left,
                    left_target,
                    catalog.ik.left_pole,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        catalog.ik.left_hand_offset_deg[0].to_radians(),
                        catalog.ik.left_hand_offset_deg[1].to_radians(),
                        catalog.ik.left_hand_offset_deg[2].to_radians(),
                    ),
                    &catalog.ik,
                    rig.root,
                    &read,
                    &parents,
                ),
            ]
        };

        let mut write = transforms.p1();
        for chain in writes.into_iter().flatten() {
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

        apply_finger_curl(&rig.arms.right, &catalog.ik.right_finger_curl, &mut write);
        apply_finger_curl(&rig.arms.left, &catalog.ik.left_finger_curl, &mut write);
    }
}

fn apply_finger_curl(chain: &ArmChain, curl: &FingerCurl, write: &mut Query<&mut Transform, ()>) {
    for (bone_entity, angles) in [
        (chain.middle_hand, curl.middle_hand_deg),
        (chain.fingers, curl.fingers_deg),
        (chain.thumb1, curl.thumb1_deg),
        (chain.thumb2, curl.thumb2_deg),
    ] {
        if angles == [0.0; 3] {
            continue;
        }
        if let Some(entity) = bone_entity
            && let Ok(mut transform) = write.get_mut(entity)
        {
            let curl_rot = Quat::from_euler(
                EulerRot::XYZ,
                angles[0].to_radians(),
                angles[1].to_radians(),
                angles[2].to_radians(),
            );
            transform.rotation = (transform.rotation * curl_rot).normalize();
        }
    }
}

fn lerp_transform(from: Transform, to: Transform, t: f32) -> Transform {
    let t = t.clamp(0.0, 1.0);
    Transform {
        translation: from.translation.lerp(to.translation, t),
        rotation: from.rotation.slerp(to.rotation, t).normalize(),
        scale: from.scale.lerp(to.scale, t),
    }
}

#[cfg(feature = "dev-ik-gizmo")]
pub fn draw_ik_gizmo(
    rigs: Query<&PlayerRig>,
    mounts: Query<&WeaponMount>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
    global_transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for mount in &mounts {
        let Some(rig) = rigs.iter().find(|rig| rig.survivor == mount.survivor) else {
            continue;
        };
        let Some(root_global) = global_transforms.get(rig.root).copied().ok() else {
            continue;
        };
        let to_world = |local: Transform| -> Vec3 { (root_global * local).translation() };

        for (chain, socket) in [
            (&rig.arms.right, mount.sockets.grip),
            (&rig.arms.left, mount.sockets.foregrip),
        ] {
            let (Some(upper), Some(mid), Some(hand)) =
                (chain.upper_arm, chain.lower_arm, chain.hand)
            else {
                continue;
            };
            let Some(upper_local) = compose_world(upper, rig.root, &transforms, &parents) else {
                continue;
            };
            let Some(mid_local) = compose_world(mid, rig.root, &transforms, &parents) else {
                continue;
            };
            let Some(hand_local) = compose_world(hand, rig.root, &transforms, &parents) else {
                continue;
            };
            let Some(socket_local) = compose_world(socket, rig.root, &transforms, &parents) else {
                continue;
            };

            let upper_pos = to_world(upper_local);
            let mid_pos = to_world(mid_local);
            let hand_pos = to_world(hand_local);
            let target_pos = to_world(socket_local);
            gizmos.line(upper_pos, mid_pos, Color::srgb(1.0, 0.5, 0.0));
            gizmos.line(mid_pos, hand_pos, Color::srgb(1.0, 0.5, 0.0));
            gizmos.line(hand_pos, target_pos, Color::srgb(0.0, 1.0, 0.5));
            gizmos.sphere(
                Isometry3d::from_translation(target_pos),
                0.05,
                Color::srgb(0.0, 1.0, 0.5),
            );
        }
    }
}

#[cfg(test)]
mod tests;
