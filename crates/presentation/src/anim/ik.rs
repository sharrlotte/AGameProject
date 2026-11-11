use bevy::prelude::*;

use crate::anim::catalog::AnimCatalog;
use crate::anim::rig::{BoneSet, PlayerRig, compose_world, parent_of};
use combat::CombatStance;
use world::{Health, Survivor};

const EPSILON: f32 = 1e-4;

/// Result of the analytic two-bone solve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub elbow: Vec3,
    pub hand: Vec3,
}

#[cfg(feature = "dev-ik-gizmo")]
pub fn draw_ik_gizmo(
    rigs: Query<&PlayerRig>,
    stance: Query<(&CombatStance, &Health), With<Survivor>>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
    global_transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for rig in &rigs {
        let Ok((combat_stance, health)) = stance.get(rig.survivor) else {
            continue;
        };
        if !combat_stance.aiming || health.current <= 0.0 || !rig.bones.is_complete() {
            continue;
        }
        let BoneSet {
            upper_arm: Some(upper),
            lower_arm: Some(mid),
            hand: Some(hand),
            target: Some(target),
        } = rig.bones
        else {
            continue;
        };

        let read = transforms;
        let Some(upper_world) = compose_world(upper, rig.root, &read, &parents) else {
            continue;
        };
        let Some(mid_world) = compose_world(mid, rig.root, &read, &parents) else {
            continue;
        };
        let Some(hand_world) = compose_world(hand, rig.root, &read, &parents) else {
            continue;
        };
        let Some(target_world) = compose_world(target, rig.root, &read, &parents) else {
            continue;
        };
        let root_global = global_transforms
            .get(rig.root)
            .copied()
            .unwrap_or(GlobalTransform::IDENTITY);

        let to_world = |local: Transform| -> Vec3 { (root_global * local).translation() };
        let upper_pos = to_world(upper_world);
        let mid_pos = to_world(mid_world);
        let hand_pos = to_world(hand_world);
        let target_pos = to_world(target_world);

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

fn rotation_is_finite(rotation: Quat) -> bool {
    rotation.is_finite() && rotation.length_squared() > EPSILON
}

/// Applies support-hand IK after animation evaluation and before propagation.
pub fn apply_support_hand_ik(
    catalog: Res<AnimCatalog>,
    rigs: Query<&PlayerRig>,
    stance: Query<(&CombatStance, &Health), With<Survivor>>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
) {
    if !catalog.ik.enabled || catalog.ik.chain.len() < 3 {
        return;
    }

    for rig in &rigs {
        let Ok((stance, health)) = stance.get(rig.survivor) else {
            continue;
        };
        if !stance.aiming || health.current <= 0.0 {
            continue;
        }
        if !rig.bones.is_complete() {
            continue;
        }

        let (upper_entity, mid_entity, hand_entity, target_entity) = match rig.bones {
            BoneSet {
                upper_arm: Some(upper),
                lower_arm: Some(mid),
                hand: Some(hand),
                target: Some(target),
            } => (upper, mid, hand, target),
            _ => continue,
        };

        let (upper_world, mid_world, hand_world, target_world, parent_world) = {
            let read = transforms.p0();
            let Some(upper_world) = compose_world(upper_entity, rig.root, &read, &parents) else {
                continue;
            };
            let Some(mid_world) = compose_world(mid_entity, rig.root, &read, &parents) else {
                continue;
            };
            let Some(hand_world) = compose_world(hand_entity, rig.root, &read, &parents) else {
                continue;
            };
            let Some(target_world) = compose_world(target_entity, rig.root, &read, &parents) else {
                continue;
            };
            let Some(parent_world) = parent_of(upper_entity, &parents)
                .and_then(|parent| compose_world(parent, rig.root, &read, &parents))
            else {
                continue;
            };
            (
                upper_world,
                mid_world,
                hand_world,
                target_world,
                parent_world,
            )
        };

        let pole_world =
            upper_world.translation + parent_world.rotation * Vec3::from_array(catalog.ik.pole);
        let Some(solution) = solve_two_bone(
            upper_world.translation,
            mid_world.translation,
            hand_world.translation,
            target_world.translation,
            pole_world - upper_world.translation,
        ) else {
            continue;
        };

        let current_upper = (mid_world.translation - upper_world.translation).normalize_or_zero();
        let desired_upper = (solution.elbow - upper_world.translation).normalize_or_zero();
        if current_upper == Vec3::ZERO || desired_upper == Vec3::ZERO {
            continue;
        }
        let upper_delta = Quat::from_rotation_arc(current_upper, desired_upper);
        let upper_world_rotation = (upper_delta * upper_world.rotation).normalize();
        let upper_local = (parent_world.rotation.inverse() * upper_world_rotation).normalize();

        let current_lower = (hand_world.translation - mid_world.translation).normalize_or_zero();
        let desired_lower = (solution.hand - solution.elbow).normalize_or_zero();
        if current_lower == Vec3::ZERO || desired_lower == Vec3::ZERO {
            continue;
        }
        let lower_delta = Quat::from_rotation_arc(current_lower, desired_lower);
        let lower_world_rotation = (lower_delta * mid_world.rotation).normalize();
        let lower_local = (upper_world_rotation.inverse() * lower_world_rotation).normalize();

        if !rotation_is_finite(upper_local) || !rotation_is_finite(lower_local) {
            continue;
        }

        let hand_local = if catalog.ik.hand_align {
            let offset = Quat::from_euler(
                EulerRot::XYZ,
                catalog.ik.hand_offset_deg[0].to_radians(),
                catalog.ik.hand_offset_deg[1].to_radians(),
                catalog.ik.hand_offset_deg[2].to_radians(),
            );
            let desired_hand = (target_world.rotation * offset).normalize();
            Some((lower_world_rotation.inverse() * desired_hand).normalize())
        } else {
            None
        };

        let mut write = transforms.p1();
        if let Ok(mut transform) = write.get_mut(upper_entity) {
            transform.rotation = upper_local;
        }
        if let Ok(mut transform) = write.get_mut(mid_entity) {
            transform.rotation = lower_local;
        }
        if let (Some(hand_local), Ok(mut transform)) = (hand_local, write.get_mut(hand_entity))
            && rotation_is_finite(hand_local)
        {
            transform.rotation = hand_local;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reach(root: Vec3, elbow: Vec3, hand: Vec3) -> (f32, f32) {
        ((elbow - root).length(), (hand - elbow).length())
    }

    #[test]
    fn solves_a_reachable_target() {
        let root = Vec3::ZERO;
        let mid = Vec3::new(0.0, 0.0, -1.0);
        let end = Vec3::new(0.0, 0.0, -2.0);
        let target = Vec3::new(0.0, 0.5, -1.4);
        let solution = solve_two_bone(root, mid, end, target, Vec3::Y).expect("solution");

        let (upper, lower) = reach(root, solution.elbow, solution.hand);
        assert!((upper - 1.0).abs() < 0.01);
        assert!((lower - 1.0).abs() < 0.01);
        assert!((solution.hand - target).length() < 0.01);
    }

    #[test]
    fn clamps_an_unreachable_target() {
        let root = Vec3::ZERO;
        let mid = Vec3::new(0.0, 0.0, -1.0);
        let end = Vec3::new(0.0, 0.0, -2.0);
        let target = Vec3::new(0.0, 0.0, -10.0);
        let solution = solve_two_bone(root, mid, end, target, Vec3::Y).expect("solution");

        assert!((solution.hand - target).length() > 1.0);
        let (upper, lower) = reach(root, solution.elbow, solution.hand);
        assert!((upper + lower - 2.0).abs() < 0.01);
    }

    #[test]
    fn rejects_degenerate_chains() {
        let root = Vec3::ZERO;
        assert!(solve_two_bone(root, root, root, Vec3::X, Vec3::Y).is_none());
        assert!(solve_two_bone(root, Vec3::Z, Vec3::ZERO, Vec3::ZERO, Vec3::Y).is_none());
    }

    #[test]
    fn mirrored_pole_vectors_produce_mirrored_elbows() {
        let root = Vec3::ZERO;
        let mid = Vec3::new(0.0, 0.0, -1.0);
        let end = Vec3::new(0.0, 0.0, -2.0);
        let target = Vec3::new(0.0, 0.0, -1.5);
        let a = solve_two_bone(root, mid, end, target, Vec3::Y).expect("solution");
        let b = solve_two_bone(root, mid, end, target, Vec3::NEG_Y).expect("solution");
        assert!((a.elbow.y + b.elbow.y).abs() < 0.001);
        assert!((a.elbow.z - b.elbow.z).abs() < 0.001);
    }
}
