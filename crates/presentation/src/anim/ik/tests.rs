use super::*;

fn reach(root: Vec3, elbow: Vec3, hand: Vec3) -> (f32, f32) {
    ((elbow - root).length(), (hand - elbow).length())
}

fn chain_target(position: Vec3) -> ArmTarget {
    ArmTarget {
        position,
        rotation: Quat::IDENTITY,
        wrist_offset: Quat::IDENTITY,
    }
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

#[test]
fn two_arms_solve_to_independent_targets() {
    let right = solve_arm(
        ChainState {
            upper_pos: Vec3::new(0.6, 1.4, 0.0),
            mid_pos: Vec3::new(0.6, 1.1, -0.4),
            hand_pos: Vec3::new(0.6, 1.0, -0.8),
            upper_rot: Quat::IDENTITY,
            mid_rot: Quat::IDENTITY,
            hand_rot: Quat::IDENTITY,
        },
        chain_target(Vec3::new(0.4, 1.2, -0.9)),
        Vec3::NEG_Y,
        true,
        Quat::IDENTITY,
    )
    .expect("right solution");

    let left = solve_arm(
        ChainState {
            upper_pos: Vec3::new(-0.6, 1.4, 0.0),
            mid_pos: Vec3::new(-0.6, 1.1, -0.4),
            hand_pos: Vec3::new(-0.6, 1.0, -0.8),
            upper_rot: Quat::IDENTITY,
            mid_rot: Quat::IDENTITY,
            hand_rot: Quat::IDENTITY,
        },
        chain_target(Vec3::new(-0.3, 1.3, -1.1)),
        Vec3::NEG_Y,
        true,
        Quat::IDENTITY,
    )
    .expect("left solution");

    assert!(right.upper_world.is_finite() && left.upper_world.is_finite());
    assert_ne!(right.upper_world, left.upper_world);
}

#[test]
fn shoulder_settle_respects_weight_and_limit() {
    let full = settle_delta(Vec3::NEG_Z, Vec3::X, 1.0, std::f32::consts::PI);
    assert!((full.angle_between(Quat::IDENTITY).to_degrees() - 90.0).abs() < 1.0);

    let zero = settle_delta(Vec3::NEG_Z, Vec3::X, 0.0, std::f32::consts::PI);
    assert_eq!(zero, Quat::IDENTITY);

    let limited = settle_delta(Vec3::NEG_Z, Vec3::X, 1.0, 0.1);
    assert!(limited.angle_between(Quat::IDENTITY) <= 0.1001);

    let half = settle_delta(Vec3::NEG_Z, Vec3::X, 0.5, std::f32::consts::PI);
    assert!((half.angle_between(Quat::IDENTITY).to_degrees() - 45.0).abs() < 1.0);
}

#[test]
fn invalid_directions_do_not_move_the_shoulder() {
    assert_eq!(settle_delta(Vec3::ZERO, Vec3::X, 1.0, 1.0), Quat::IDENTITY);
    assert_eq!(
        settle_delta(Vec3::NEG_Z, Vec3::ZERO, 1.0, 1.0),
        Quat::IDENTITY
    );
}
