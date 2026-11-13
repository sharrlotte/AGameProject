use bevy::prelude::*;

use super::resolve::{ResolvedGrip, ResolvedHand};

/// An orthonormal frame describing a hand's orientation.
///
/// `+Y` runs from the wrist to the knuckles, `+Z` points out of the back of the
/// hand and `+X` completes the right-handed frame. On the right hand `+X`
/// points towards the little-finger side, on the left hand towards the thumb.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandFrame {
    pub side: Vec3,
    pub fingers: Vec3,
    pub dorsal: Vec3,
}

impl HandFrame {
    /// Builds the frame from the authored finger and palm directions.
    ///
    /// The palm direction is the direction the palm presses into the grip, so
    /// the dorsal axis is its opposite. The palm direction is orthogonalised
    /// against the finger direction, which lets an author describe the palm
    /// with a rough direction instead of an exact one.
    pub fn from_directions(finger: Vec3, palm: Vec3) -> Option<Self> {
        let fingers = finger.normalize();
        if !fingers.is_finite() || fingers == Vec3::ZERO {
            return None;
        }
        let dorsal = -(palm - fingers * palm.dot(fingers));
        let dorsal = dorsal.normalize();
        if !dorsal.is_finite() || dorsal == Vec3::ZERO {
            return None;
        }
        let side = fingers.cross(dorsal);
        if !side.is_finite() || side.length_squared() < 1e-6 {
            return None;
        }
        Some(Self {
            side: side.normalize(),
            fingers,
            dorsal,
        })
    }

    /// The rotation that maps the hand bone's local axes onto this frame.
    pub fn rotation(&self) -> Quat {
        Quat::from_mat3(&Mat3::from_cols(self.side, self.fingers, self.dorsal)).normalize()
    }
}

/// One hand's grip target, expressed in the weapon's own space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GripTarget {
    /// Where the palm bone origin has to sit.
    pub position: Vec3,
    /// How the hand has to be oriented.
    pub frame: HandFrame,
    /// Direction the elbow bends towards.
    pub elbow: Vec3,
    /// Extra rotation in weapon space applied after the palm frame.
    pub align: Quat,
    /// Extra rotation in the hand bone's own frame, when authored.
    pub wrist: Option<Quat>,
}

impl GripTarget {
    /// The full hand rotation in weapon space, including the alignment offset.
    pub fn rotation(&self) -> Quat {
        (self.align * self.frame.rotation()).normalize()
    }

    /// The authored wrist trim, falling back to the catalog offset.
    pub fn wrist(&self, fallback: Quat) -> Quat {
        self.wrist.unwrap_or(fallback)
    }

    fn from_hand(hand: &ResolvedHand) -> Option<Self> {
        Some(Self {
            position: hand.position,
            frame: HandFrame::from_directions(hand.finger, hand.palm)?,
            elbow: hand.elbow,
            align: hand.align,
            wrist: hand.wrist,
        })
    }
}

/// The grip targets of one weapon, expressed in the weapon's own space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponTargets {
    pub primary: GripTarget,
    /// The support hand, when the weapon is held with two hands.
    pub support: Option<GripTarget>,
}

impl WeaponTargets {
    /// Builds the targets of a resolved grip, in weapon space.
    pub fn from_grip(grip: &ResolvedGrip) -> Option<Self> {
        Some(Self {
            primary: GripTarget::from_hand(&grip.primary)?,
            support: match grip.support {
                Some(hand) => Some(GripTarget::from_hand(&hand)?),
                None => None,
            },
        })
    }
}







