use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use combat::WeaponInventory;

use crate::anim::PlayerVisualRoot;

use super::catalog::WeaponCatalog;
use super::state::{PlayerWeaponAnchor, ResolvedSockets, WeaponMount, WeaponPoseState};

/// Tracks an in-flight weapon GLTF load so its handle stays alive until the asset
/// is ready (Bevy unloads assets with no live handles).
#[derive(Component)]
pub struct PendingWeaponLoad {
    pub handle: Handle<Gltf>,
    pub profile_id: String,
}

/// Spawns the chest-height weapon anchor under every new player visual root.
pub fn init_weapon_anchors(
    mut commands: Commands,
    catalog: Res<WeaponCatalog>,
    roots: Query<(Entity, &ChildOf), Added<PlayerVisualRoot>>,
) {
    for (root, child_of) in &roots {
        commands.spawn((
            Name::new("WeaponAnchor"),
            PlayerWeaponAnchor {
                root,
                survivor: child_of.parent(),
            },
            anchor_transform(&catalog),
            WeaponPoseState::default(),
            ChildOf(root),
            Visibility::default(),
        ));
    }
}

fn anchor_transform(catalog: &WeaponCatalog) -> Transform {
    Transform::from_xyz(
        catalog.anchor.offset[0],
        catalog.anchor.offset[1],
        catalog.anchor.offset[2],
    )
    .with_rotation(super::state::euler_deg(catalog.anchor.rotation_deg))
}

/// Mounts the active weapon profile and remounts on swap or catalog reload.
pub fn sync_weapon_mounts(
    mut commands: Commands,
    catalog: Res<WeaponCatalog>,
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
    let reloaded = catalog.is_changed();

    for (anchor_entity, anchor, mut transform, mount, pending, mut pose) in &mut anchors {
        *transform = anchor_transform(&catalog);
        if reloaded {
            *pose = WeaponPoseState::default();
        }

        let Ok(inventory) = inventories.get(anchor.survivor) else {
            continue;
        };
        let Some(slot) = inventory.slots.get(inventory.active_slot) else {
            continue;
        };
        let profile_id = slot.spec.id.as_str();

        // Despawn + clear mount/pending on profile change or reload.
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

        // If a pending load exists for this profile, check whether it finished.
        if let Some(pending) = pending
            && pending.profile_id == profile_id
            && !reloaded
        {
            if let Some(gltf) = gltfs.get(&pending.handle) {
                let Some(scene) = gltf.scenes.first().cloned() else {
                    warn!("Weapon '{}' contains no scenes", profile_id);
                    commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
                    continue;
                };
                let profile = match catalog.weapon(profile_id) {
                    Some(p) => p,
                    None => {
                        warn!("No presentation profile for weapon '{profile_id}'");
                        commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
                        continue;
                    }
                };
                let scene_entity = commands
                    .spawn((
                        Name::new(format!("WeaponModel:{}", profile.id)),
                        WorldAssetRoot(scene),
                        Transform::default(),
                        Visibility::default(),
                        ChildOf(anchor_entity),
                    ))
                    .id();
                commands.entity(anchor_entity).insert(WeaponMount {
                    survivor: anchor.survivor,
                    root: anchor.root,
                    profile: profile.id.clone(),
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

        let Some(profile) = catalog.weapon(profile_id) else {
            warn!("No presentation profile for weapon '{profile_id}'");
            continue;
        };

        // Start a new load, keeping the handle alive in a pending component.
        let handle = asset_server.load(profile.model.clone());
        commands.entity(anchor_entity).insert(PendingWeaponLoad {
            handle,
            profile_id: profile.id.clone(),
        });
    }
}

/// Resolves `Grip`/`Foregrip`/`Magazine`/`Muzzle` once per mounted weapon.
pub fn resolve_weapon_sockets(
    catalog: Res<WeaponCatalog>,
    mut mounts: Query<&mut WeaponMount>,
    children: Query<&Children>,
    names: Query<&Name>,
    transforms: Query<&Transform>,
) {
    for mut mount in &mut mounts {
        if mount.resolved {
            continue;
        }
        let Some(profile) = catalog.weapon(&mount.profile) else {
            mount.resolved = true;
            continue;
        };
        let Ok(root_children) = children.get(mount.scene) else {
            continue;
        };
        if root_children.is_empty() {
            continue;
        }

        let wanted = profile.sockets.named();
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

        mount.magazine_rest = magazine
            .and_then(|entity| transforms.get(entity).ok().copied())
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
