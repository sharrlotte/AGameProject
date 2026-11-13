use bevy::prelude::*;

use super::*;

fn sockets() -> SocketFrames {
    SocketFrames {
        grip: Some(Transform::IDENTITY),
        foregrip: Some(Transform::from_xyz(0.0, 0.0, -0.3)),
        magazine: Some(Transform::from_xyz(0.0, -0.08, 0.0)),
        muzzle: Some(Transform::from_xyz(0.0, 0.0, -1.0)),
    }
}

#[test]
fn authored_fields_override_socket_defaults_field_by_field() {
    let authored = WeaponGripConfig {
        primary: GripHand {
            position: Some([0.1, 0.2, 0.3]),
            ..GripHand::default()
        },
        support: Some(GripHand::default()),
        ..WeaponGripConfig::default()
    };

    let grip = resolve_grip(Some(&authored), &sockets());

    assert_eq!(grip.primary.position, Vec3::new(0.1, 0.2, 0.3));
    assert!(grip.primary.finger.length_squared() > 0.0);
    assert!(grip.primary.palm.length_squared() > 0.0);
    assert!(grip.support.is_some());
}

#[test]
fn derived_support_uses_the_foregrip_socket() {
    let grip = resolve_grip(None, &sockets());
    let support = grip.support.expect("two-handed grip");
    let foregrip = Vec3::new(0.0, 0.0, -0.3);

    assert!((support.position - foregrip).length() < 0.06);
    assert!(support.finger.z < 0.0);
}

#[test]
fn close_foregrip_does_not_create_a_support_hand() {
    let mut sockets = sockets();
    sockets.foregrip = Some(Transform::from_xyz(0.0, 0.0, -0.05));

    let grip = resolve_grip(None, &sockets);
    assert!(grip.support.is_none());
}

#[test]
fn authored_support_survives_a_missing_foregrip() {
    let mut sockets = sockets();
    sockets.foregrip = None;
    let authored = WeaponGripConfig {
        support: Some(GripHand {
            position: Some([0.0, 0.0, -0.2]),
            ..GripHand::default()
        }),
        ..WeaponGripConfig::default()
    };

    let grip = resolve_grip(Some(&authored), &sockets);
    let support = grip.support.expect("authored support");
    assert_eq!(support.position, Vec3::new(0.0, 0.0, -0.2));
}

#[test]
fn missing_grip_socket_falls_back_to_the_weapon_origin() {
    let sockets = SocketFrames::default();
    let grip = resolve_grip(None, &sockets);

    assert_eq!(grip.primary.position, Vec3::new(0.028, -0.032, 0.0));
    assert!(grip.support.is_none());
}

#[test]
fn socket_rotation_turns_the_derived_basis() {
    let mut sockets = sockets();
    sockets.grip = Some(Transform::from_rotation(Quat::from_rotation_y(
        std::f32::consts::FRAC_PI_2,
    )));
    sockets.muzzle = None;

    let grip = resolve_grip(None, &sockets);

    assert!((grip.primary.palm - Vec3::NEG_X).length() > 0.1);
}

#[test]
fn authored_wrist_and_align_are_carried_through() {
    let authored = WeaponGripConfig {
        primary: GripHand {
            wrist_deg: Some([0.0, 0.0, 15.0]),
            align_deg: Some([5.0, 0.0, 0.0]),
            ..GripHand::default()
        },
        ..WeaponGripConfig::default()
    };

    let grip = resolve_grip(Some(&authored), &sockets());
    assert!(grip.primary.wrist.is_some());
    assert!((grip.primary.align.angle_between(Quat::IDENTITY) - 5.0f32.to_radians()).abs() < 1e-3);
}

#[test]
fn curl_falls_back_to_the_catalog_default() {
    let grip = resolve_grip(None, &sockets());
    let fallback = FingerCurl {
        middle_hand_deg: 10.0,
        ..FingerCurl::default()
    };

    assert_eq!(resolve_curl(&grip.primary, fallback), fallback);
}
