use bevy::prelude::*;
use crate::combat::{CombatStance, WeaponInventory};
use crate::message::messages::WeaponFiredMessage;
use crate::message::messages::WeaponSwappedMessage;
use crate::world::{Health, Survivor};

use crate::presentation::anim::blend::approach;
use crate::presentation::anim::catalog::AnimCatalog;

use super::state::{
    PlayerWeaponAnchor, WeaponMount, WeaponPoseState, add_kick, compose_weapon, decay_kick,
    reload_offset, swap_curve,
};

const MAX_POSES: usize = 8;

#[derive(Clone, Copy)]
struct PoseWrite {
    scene: Entity,
    magazine: Option<(Entity, Transform)>,
    weapon: Transform,
}

impl Default for PoseWrite {
    fn default() -> Self {
        Self {
            scene: Entity::PLACEHOLDER,
            magazine: None,
            weapon: Transform::IDENTITY,
        }
    }
}

/// Adds a clamped recoil impulse when the player fires.
pub fn update_weapon_kick(
    mut fired: MessageReader<WeaponFiredMessage>,
    weapons: Query<&WeaponInventory>,
    mut mounts: Query<(&WeaponMount, &mut WeaponPoseState)>,
) {
    for message in fired.read() {
        let Some((mount, mut state)) = mounts
            .iter_mut()
            .find(|(mount, _)| mount.survivor == message.shooter)
        else {
            continue;
        };
        let Ok(inventory) = weapons.get(mount.survivor) else {
            continue;
        };
        let Some(slot) = inventory.slots.get(inventory.active_slot) else {
            continue;
        };
        add_kick(&mut state, &slot.weapon);
    }
}

/// Starts the lower/raise timeline when the player swaps weapons.
pub fn update_weapon_swap(
    mut swapped: MessageReader<WeaponSwappedMessage>,
    inventories: Query<&WeaponInventory>,
    mut anchors: Query<(&PlayerWeaponAnchor, &mut WeaponPoseState)>,
) {
    for message in swapped.read() {
        let Ok(inventory) = inventories.get(message.entity) else {
            continue;
        };
        let Some(slot) = inventory.slots.get(message.to) else {
            continue;
        };
        let duration = slot.weapon.presentation.swap.lower_secs
            + slot.weapon.presentation.swap.raise_secs;
        let Some((_, mut state)) = anchors
            .iter_mut()
            .find(|(anchor, _)| anchor.survivor == message.entity)
        else {
            continue;
        };
        state.swap_elapsed = 0.0;
        state.swap_duration = duration;
    }
}

/// Poses the mounted weapon after animation evaluation, before arm IK.
#[allow(clippy::type_complexity)]
pub fn pose_player_weapon(
    time: Res<Time>,
    anim: Res<AnimCatalog>,
    survivors: Query<(&Health, &CombatStance, &WeaponInventory), With<Survivor>>,
    mut param: ParamSet<(
        Query<(&PlayerWeaponAnchor, &mut WeaponPoseState, &WeaponMount)>,
        Query<&mut Transform>,
    )>,
) {
    let dt = time.delta_secs();
    let mut writes = [PoseWrite::default(); MAX_POSES];
    let mut count = 0;

    {
        for (anchor, mut state, mount) in &mut param.p0() {
            let Ok((health, stance, inventory)) = survivors.get(anchor.survivor) else {
                continue;
            };
            if health.current <= 0.0 {
                continue;
            }
            let weapon = inventory
                .slots
                .get(inventory.active_slot)
                .map(|slot| &slot.weapon)
                .unwrap_or_else(|| &inventory.slots[0].weapon);

            state.aim = approach(
                state.aim,
                if stance.aiming { 1.0 } else { 0.0 },
                dt,
                anim.aim.blend_secs,
            );
            decay_kick(&mut state, dt, weapon);
            state.sway = (state.sway + dt * weapon.presentation.sway.frequency * std::f32::consts::TAU)
                .rem_euclid(std::f32::consts::TAU);
            state.swap_elapsed = (state.swap_elapsed + dt).min(state.swap_duration);

            let reload = reload_offset(inventory.reload_progress(), &weapon.presentation.reload);
            let swap = swap_curve(state.swap_elapsed, state.swap_duration);
            let composed = compose_weapon(weapon, &state, reload, swap);
            state.frame = composed;

            let magazine = mount.sockets.magazine.map(|entity| {
                let mut rest = mount.magazine_rest;
                rest.translation.y -= reload.mag_drop;
                rest.translation.z += reload.mag_drop * 0.2;
                (entity, rest)
            });

            if count < MAX_POSES {
                writes[count] = PoseWrite {
                    scene: mount.scene,
                    magazine,
                    weapon: composed.mul_transform(weapon.model_transform()),
                };
                count += 1;
            }
        }
    }

    let mut transforms = param.p1();
    for write in &writes[..count] {
        if let Ok(mut transform) = transforms.get_mut(write.scene) {
            *transform = write.weapon;
        }
        if let Some((entity, magazine)) = write.magazine
            && let Ok(mut transform) = transforms.get_mut(entity)
        {
            *transform = magazine;
        }
    }
}
