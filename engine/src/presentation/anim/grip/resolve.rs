use bevy::prelude::*;

use super::config::{FingerCurl, GripHand, WeaponGripConfig};

/// Socket frames of a mounted weapon, in the weapon's own space.
#[derive(Debug, Clone, Copy, Default)]
pub struct SocketFrames {
    pub grip: Option<Transform>,
    pub foregrip: Option<Transform>,
    pub magazine: Option<Transform>,
    pub muzzle: Option<Transform>,
}

/// A hand's grip after authored values have been merged with the socket-derived
/// defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedHand {
    pub position: Vec3,
    pub finger: Vec3,
    pub palm: Vec3,
    pub elbow: Vec3,
    pub fingers: Option<FingerCurl>,
    pub wrist: Option<Quat>,
    pub align: Quat,
}

/// A grip resolved for one weapon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedGrip {
    pub primary: ResolvedHand,
    pub support: Option<ResolvedHand>,
}

/// Resolves the grip of a weapon.
///
/// Authored values win field by field. Omitted values fall back to a grip
/// derived from the weapon's socket frames, so adding a gun to the catalog
/// never requires an engine change.
pub fn resolve_grip(authored: Option<&WeaponGripConfig>, sockets: &SocketFrames) -> ResolvedGrip {
    let min_separation = authored
        .map(|grip| grip.min_support_separation)
        .unwrap_or(0.08);
    let basis = weapon_basis(sockets);
    let grip_position = sockets
        .grip
        .map(|frame| frame.translation)
        .unwrap_or(Vec3::ZERO);
    let derived_primary = derive_primary(&basis, grip_position);
    let derived_support = derive_support(sockets, &basis, min_separation);

    let primary = match authored {
        Some(grip) => merge_hand(grip.primary, derived_primary),
        None => derived_primary,
    };
    let support = match authored {
        Some(grip) => grip.support.map(|hand| {
            let base = derived_support.unwrap_or_else(|| safe_support(&basis, grip_position));
            merge_hand(hand, base)
        }),
        None => derived_support,
    };

    ResolvedGrip { primary, support }
}

/// The weapon's forward, up and right axes in weapon space.
#[derive(Debug, Clone, Copy)]
struct Basis {
    forward: Vec3,
    up: Vec3,
    right: Vec3,
}

fn weapon_basis(sockets: &SocketFrames) -> Basis {
    let rotation = sockets
        .grip
        .map(|frame| frame.rotation)
        .unwrap_or(Quat::IDENTITY);
    let mut forward = rotation * Vec3::NEG_Z;

    // The muzzle position is the most reliable forward hint when a socket frame
    // has no authored rotation.
    if let (Some(grip), Some(muzzle)) = (sockets.grip, sockets.muzzle) {
        let delta = muzzle.translation - grip.translation;
        if delta.length_squared() > 1e-8 {
            forward = delta.normalize();
        }
    }
    let forward = normalized_or(forward, Vec3::NEG_Z);
    let up = orthogonalize(rotation * Vec3::Y, forward);
    let right = normalized_or(forward.cross(up), Vec3::X);
    let up = normalized_or(right.cross(forward), Vec3::Y);
    Basis { forward, up, right }
}

fn derive_primary(basis: &Basis, grip: Vec3) -> ResolvedHand {
    ResolvedHand {
        position: grip + basis.right * 0.028 - basis.up * 0.032,
        finger: normalized_or(basis.up - basis.forward * 0.25, basis.up),
        palm: -basis.right,
        elbow: normalized_or(-basis.up + basis.right * 0.6, Vec3::NEG_Y),
        fingers: None,
        wrist: None,
        align: Quat::IDENTITY,
    }
}

fn derive_support(
    sockets: &SocketFrames,
    basis: &Basis,
    min_separation: f32,
) -> Option<ResolvedHand> {
    let grip = sockets.grip.map(|frame| frame.translation).unwrap_or(Vec3::ZERO);
    let foregrip = sockets.foregrip?;
    if foregrip.translation.distance(grip) < min_separation.max(0.0) {
        return None;
    }
    Some(support_at(foregrip.translation, basis))
}

fn safe_support(basis: &Basis, grip: Vec3) -> ResolvedHand {
    support_at(grip + basis.forward * 0.15, basis)
}

fn support_at(position: Vec3, basis: &Basis) -> ResolvedHand {
    ResolvedHand {
        position: position - basis.right * 0.032 - basis.up * 0.026,
        finger: normalized_or(basis.forward + basis.right * 0.55, basis.forward),
        palm: normalized_or(basis.right * 0.6 + basis.up * 0.8, basis.right),
        elbow: normalized_or(-basis.up - basis.right * 0.6, Vec3::NEG_Y),
        fingers: None,
        wrist: None,
        align: Quat::IDENTITY,
    }
}

fn merge_hand(authored: GripHand, base: ResolvedHand) -> ResolvedHand {
    ResolvedHand {
        position: authored.position().unwrap_or(base.position),
        finger: authored.finger().unwrap_or(base.finger),
        palm: authored.palm().unwrap_or(base.palm),
        elbow: authored.elbow().unwrap_or(base.elbow),
        fingers: authored.fingers.or(base.fingers),
        wrist: authored.wrist().or(base.wrist),
        align: authored.align().unwrap_or(base.align),
    }
}

/// Removes the component of `vector` along `axis`.
fn orthogonalize(vector: Vec3, axis: Vec3) -> Vec3 {
    let projected = vector - axis * vector.dot(axis);
    if projected.length_squared() < 1e-6 {
        return Vec3::Y;
    }
    projected.normalize()
}

fn normalized_or(vector: Vec3, fallback: Vec3) -> Vec3 {
    let value = vector.normalize_or_zero();
    if value == Vec3::ZERO {
        fallback
    } else {
        value
    }
}

/// The finger curl applied to a hand, falling back to the catalog default.
pub fn resolve_curl(hand: &ResolvedHand, fallback: FingerCurl) -> FingerCurl {
    hand.fingers.unwrap_or(fallback)
}







