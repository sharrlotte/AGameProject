use bevy::prelude::*;
use combat::{CombatStance, WeaponInventory};
use data::{WeaponFiredMessage, WeaponSwappedMessage};
use world::{Health, Survivor};

use crate::anim::blend::approach;
use crate::anim::catalog::AnimCatalog;

use super::catalog::WeaponCatalog;
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
    catalog: Res<WeaponCatalog>,
    mut mounts: Query<(&WeaponMount, &mut WeaponPoseState)>,
) {
    for message in fired.read() {
        let Some((mount, mut state)) = mounts
            .iter_mut()
            .find(|(mount, _)| mount.survivor == message.shooter)
        else {
            continue;
        };
        let Some(profile) = catalog.weapon(&mount.profile) else {
            continue;
        };
        add_kick(&mut state, profile);
    }
}

/// Starts the lower/raise timeline when the player swaps weapons.
pub fn update_weapon_swap(
    mut swapped: MessageReader<WeaponSwappedMessage>,
    catalog: Res<WeaponCatalog>,
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
        let Some(profile) = catalog.weapon(&slot.spec.id) else {
            continue;
        };
        let duration = profile.swap.total_secs();
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
    catalog: Res<WeaponCatalog>,
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
            let Some(profile) = catalog.weapon(&mount.profile) else {
                continue;
            };

            state.aim = approach(
                state.aim,
                if stance.aiming { 1.0 } else { 0.0 },
                dt,
                anim.aim.blend_secs,
            );
            decay_kick(&mut state, dt, profile);
            state.sway = (state.sway + dt * profile.sway.frequency * std::f32::consts::TAU)
                .rem_euclid(std::f32::consts::TAU);
            state.swap_elapsed = (state.swap_elapsed + dt).min(state.swap_duration);

            let reload = reload_offset(inventory.reload_progress(), &profile.reload);
            let swap = swap_curve(state.swap_elapsed, state.swap_duration);
            let weapon = compose_weapon(profile, &state, reload, swap);
            state.frame = weapon;

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
                    weapon: weapon.mul_transform(profile.model_transform()),
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
