use bevy::prelude::*;

use crate::presentation::anim::grip::SocketFrames;
use crate::presentation::anim::rig::compose_world;

use super::state::WeaponMount;

/// Socket frames of a mounted weapon, expressed in the weapon's own space.
///
/// The weapon's own space is the one the grip configurations are authored in:
/// `+Y` up, `-Z` towards the muzzle and `+X` towards the weapon's right. The
/// weapon frame carries no scale, so the inverse is a rigid transform.
pub fn socket_frames(
    mount: &WeaponMount,
    weapon_frame: &Transform,
    transforms: &Query<&Transform>,
    parents: &Query<&ChildOf>,
    root: Entity,
) -> SocketFrames {
    let to_weapon_space = |entity: Entity| -> Option<Transform> {
        let frame = compose_world(entity, root, transforms, parents)?;
        let translation = weapon_frame
            .compute_affine()
            .inverse()
            .transform_point3(frame.translation);
        let rotation = (weapon_frame.rotation.inverse() * frame.rotation).normalize();
        Some(Transform::from_translation(translation).with_rotation(rotation))
    };

    SocketFrames {
        grip: to_weapon_space(mount.sockets.grip),
        foregrip: to_weapon_space(mount.sockets.foregrip),
        magazine: mount.sockets.magazine.and_then(to_weapon_space),
        muzzle: mount.sockets.muzzle.and_then(to_weapon_space),
    }
}







