//! Weapon grip configuration.
//!
//! A grip describes how the character holds a weapon: where each palm sits on
//! the weapon, which way the palm faces, which way the fingers point, and
//! which way the elbow is allowed to bend.
//!
//! # Coordinate spaces
//!
//! * **Weapon space** — the space a weapon is mounted in. `+Y` is up, `-Z` is
//!   the muzzle direction and `+X` is the weapon's right-hand side. The
//!   character in `PlayerVisualRoot` space uses the same axes, so a weapon
//!   mounted with an identity rotation already points where the body faces.
//! * **Character space** — `PlayerVisualRoot` space: `+Y` up, `-Z` forward,
//!   `+X` to the character's right.
//!
//! Grip values are authored in weapon space so a single configuration stays
//! valid for idle, walking, strafing and aiming: the weapon moves, the hands
//! follow.

mod config;
mod resolve;
mod target;

#[cfg(test)]
mod tests;

pub use config::{FingerCurl, GripHand, WeaponGripConfig};
pub use resolve::{
    ResolvedGrip, ResolvedHand, SocketFrames, resolve_curl, resolve_grip,
};
pub use target::{GripTarget, HandFrame, WeaponTargets};







