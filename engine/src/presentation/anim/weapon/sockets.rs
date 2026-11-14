use crate::config::weapons::WeaponDefinition;
use crate::presentation::anim::grip::SocketFrames;

/// Socket frames of a weapon, expressed directly in the weapon's own space.
///
/// The weapon's own space is the one the grip and socket configurations are authored in:
/// `+Y` up, `-Z` towards the muzzle and `+X` towards the weapon's right.
pub fn socket_frames(weapon: &WeaponDefinition) -> SocketFrames {
    SocketFrames {
        grip: Some(weapon.hands.primary.transform()),
        foregrip: weapon.hands.support.as_ref().map(|s| s.transform()),
        magazine: Some(weapon.sockets.magazine.transform()),
        muzzle: Some(weapon.sockets.muzzle.transform()),
    }
}
