use bevy::prelude::*;

use crate::anim::catalog::IkConfig;
use crate::anim::rig::{ArmChain, compose_world, parent_of};

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
    /// Trims the solved hand rotation in the hand bone's own frame.
    pub wrist_offset: Quat,
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

/// Local rotations to write back for one solved arm.
pub struct ChainWrites {
    pub shoulder: Option<(Entity, Quat)>,
    pub upper: (Entity, Quat),
    pub lower: (Entity, Quat),
    pub hand: Option<(Entity, Quat)>,
}

/// Solves elbow and hand positions for a two-bone chain.
///
/// `root` is the shoulder, `mid` the elbow, `end` the wrist. `pole_hint` is a
/// world-space direction the elbow is allowed to bend towards. Unreachable
/// targets are clamped to the chain's reach.
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
    wrist_offset: Quat,
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
        (target.rotation * wrist_offset).normalize()
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

/// Solves one arm chain and returns the local rotations to write.
///
/// `pole` is a world-space direction the elbow bends towards. The target's
/// `wrist_offset` trims the solved hand rotation in the hand bone's own frame.
pub fn solve_chain(
    chain: &ArmChain,
    target: ArmTarget,
    pole: Vec3,
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
        pole,
        ik.hand_align,
        target.wrist_offset,
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
