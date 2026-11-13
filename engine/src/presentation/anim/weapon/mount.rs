use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use crate::combat::WeaponInventory;

use crate::presentation::anim::PlayerVisualRoot;

use crate::config::WeaponRoster;
use super::state::{PlayerWeaponAnchor, ResolvedSockets, WeaponMount, WeaponPoseState};

/// Tracks an in-flight weapon GLTF load so its handle stays alive until the asset
/// is ready (Bevy unloads assets with no live handles).
#[derive(Component)]
pub struct PendingWeaponLoad {
    pub handle: Handle<Gltf>,
    pub profile_id: String,
    pub failed: bool,
}

/// Spawns the chest-height weapon anchor under every new player visual root.
pub fn init_weapon_anchors(
    mut commands: Commands,
    roster: Res<WeaponRoster>,
    roots: Query<(Entity, &ChildOf), Added<PlayerVisualRoot>>,
) {
    for (root, child_of) in &roots {
        commands.spawn((
            Name::new("WeaponAnchor"),
            PlayerWeaponAnchor {
                root,
                survivor: child_of.parent(),
            },
            anchor_transform(&roster),
            WeaponPoseState::default(),
            ChildOf(root),
            Visibility::default(),
        ));
    }
}

fn anchor_transform(roster: &WeaponRoster) -> Transform {
    Transform::from_xyz(
        roster.anchor.offset[0],
        roster.anchor.offset[1],
        roster.anchor.offset[2],
    )
    .with_rotation(super::state::euler_deg(roster.anchor.rotation_deg))
}

/// Mounts the active weapon and remounts on swap or roster reload.
pub fn sync_weapon_mounts(
    mut commands: Commands,
    roster: Res<WeaponRoster>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    inventories: Query<&WeaponInventory>,
    mut anchors: Query<(
        Entity,
        &PlayerWeaponAnchor,
        &mut Transform,
        Option<&WeaponMount>,
        Option<&PendingWeaponLoad>,
        &mut WeaponPoseState,
    )>,
) {
    let reloaded = roster.is_changed();

    for (anchor_entity, anchor, mut transform, mount, pending, mut pose) in &mut anchors {
        let weapon = inventories
            .get(anchor.survivor)
            .ok()
            .and_then(|inventory| {
                inventory
                    .slots
                    .get(inventory.active_slot)
                    .map(|slot| &slot.weapon)
            });
        let Some(weapon) = weapon else {
            continue;
        };
        let profile_id = weapon.id.clone();

        *transform = anchor_transform(&roster);
        if reloaded {
            *pose = WeaponPoseState::default();
        }

        // Despawn + clear mount/pending on weapon change or reload.
        if let Some(mount) = mount {
            if !reloaded && mount.profile == profile_id {
                // Already mounted and up to date; keep handle alive via pending? No pending exists.
                continue;
            }
            commands.entity(mount.scene).despawn();
            commands.entity(anchor_entity).remove::<WeaponMount>();
        }
        if let Some(pending) = pending
            && (reloaded || pending.profile_id != profile_id)
        {
            commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
        }

        // If a pending load exists for this weapon, check whether it finished.
        if let Some(pending) = pending
            && pending.profile_id == profile_id
            && !reloaded
        {
            if pending.failed {
                continue;
            }
            if let Some(gltf) = gltfs.get(&pending.handle) {
                let Some(scene) = gltf.scenes.first().cloned() else {
                    warn!("Weapon '{}' contains no scenes", profile_id);
                    commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
                    commands.entity(anchor_entity).insert(PendingWeaponLoad {
                        handle: pending.handle.clone(),
                        profile_id: pending.profile_id.clone(),
                        failed: true,
                    });
                    continue;
                };
                let scene_entity = commands
                    .spawn((
                        Name::new(format!("WeaponModel:{}", weapon.id)),
                        WorldAssetRoot(scene),
                        Transform::default(),
                        Visibility::default(),
                        ChildOf(anchor_entity),
                    ))
                    .id();
                commands.entity(anchor_entity).insert(WeaponMount {
                    survivor: anchor.survivor,
                    root: anchor.root,
                    profile: weapon.id.clone(),
                    scene: scene_entity,
                    sockets: ResolvedSockets {
                        grip: scene_entity,
                        foregrip: scene_entity,
                        magazine: None,
                        muzzle: None,
                    },
                    magazine_rest: Transform::IDENTITY,
                    resolved: false,
                });
                commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
            }
            // Still loading: keep pending (handle stays alive).
            continue;
        }

        // Start a new load, keeping the handle alive in a pending component.
        let handle = asset_server.load(&weapon.presentation.model);
        commands.entity(anchor_entity).insert(PendingWeaponLoad {
            handle,
            profile_id: weapon.id.clone(),
            failed: false,
        });
    }
}

/// Resolves `Grip`/`Foregrip`/`Magazine`/`Muzzle` once per mounted weapon.
pub fn resolve_weapon_sockets(
    roster: Res<WeaponRoster>,
    mut mounts: Query<&mut WeaponMount>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut socket_transforms: Query<&mut Transform>,
) {
    for mut mount in &mut mounts {
        if mount.resolved {
            continue;
        }
        let Some(weapon) = roster.weapon(&mount.profile) else {
            mount.resolved = true;
            continue;
        };
        let Ok(root_children) = children.get(mount.scene) else {
            continue;
        };
        if root_children.is_empty() {
            continue;
        }

        let wanted = weapon.presentation.sockets.named();
        let mut found: [Option<Entity>; 4] = [None; 4];
        let mut stack: Vec<Entity> = root_children.iter().collect();
        while let Some(entity) = stack.pop() {
            if let Ok(name) = names.get(entity) {
                for (index, socket) in wanted.iter().enumerate() {
                    if found[index].is_none() && name.as_str() == *socket {
                        found[index] = Some(entity);
                    }
                }
            }
            if let Ok(children) = children.get(entity) {
                stack.extend(children.iter());
            }
        }

        let [grip, foregrip, magazine, muzzle] = found;
        let mut missing = Vec::new();
        if grip.is_none() {
            missing.push(wanted[0]);
        }
        if foregrip.is_none() {
            missing.push(wanted[1]);
        }
        if magazine.is_none() {
            missing.push(wanted[2]);
        }
        if muzzle.is_none() {
            missing.push(wanted[3]);
        }
        if !missing.is_empty() {
            warn!(
                "Weapon '{}' is missing sockets {:?}; falling back to the weapon root",
                mount.profile, missing
            );
        }

        // Authored socket placements replace the ones baked into the model.
        for (index, socket_name) in wanted.iter().enumerate() {
            let Some(entity) = found[index] else {
                continue;
            };
            let Some(over) = weapon
                .presentation
                .socket_overrides
                .iter()
                .find(|over| over.socket == *socket_name)
            else {
                continue;
            };
            if let Ok(mut transform) = socket_transforms.get_mut(entity) {
                *transform = over.transform();
            }
        }

        mount.magazine_rest = magazine
            .and_then(|entity| socket_transforms.get(entity).ok().copied())
            .unwrap_or(Transform::IDENTITY);
        mount.sockets = ResolvedSockets {
            grip: grip.unwrap_or(mount.scene),
            foregrip: foregrip.unwrap_or(mount.scene),
            magazine,
            muzzle,
        };
        mount.resolved = true;
    }
}
