use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use crate::config::WeaponRoster;
use crate::presentation::anim::PlayerVisualRoot;
use super::state::{
    PlayerWeaponAnchor, PlayerWeaponRuntime, ResolvedSockets, WeaponMount,
    WeaponPoseState,
};

/// Tracks an in-flight weapon GLTF load so its handle stays alive until the asset
/// is ready (Bevy unloads assets with no live handles).
#[derive(Component)]
pub struct PendingWeaponLoad {
    pub handle: Handle<Gltf>,
    pub profile_id: String,
    pub failed: bool,
}

/// Places the weapon root under the player's chest anchor.
pub fn anchor_transform(roster: &WeaponRoster) -> Transform {
    let anchor = &roster.anchor;
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        anchor.rotation_deg[0].to_radians(),
        anchor.rotation_deg[1].to_radians(),
        anchor.rotation_deg[2].to_radians(),
    )
    .normalize();
    Transform::from_translation(Vec3::from_array(anchor.offset)).with_rotation(rotation)
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
            PlayerWeaponRuntime {
                weapon: roster.weapons.first().cloned().unwrap_or_default(),
            },
            WeaponPoseState::default(),
            ChildOf(root),
            Visibility::default(),
        ));
    }
}

/// Spawns and tracks weapon scenes as the active profile changes.
pub fn sync_weapon_mounts(
    mut commands: Commands,
    roster: Res<WeaponRoster>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut anchors: Query<(
        Entity,
        &PlayerWeaponAnchor,
        &mut Transform,
        Option<&WeaponMount>,
        Option<&PendingWeaponLoad>,
        &mut WeaponPoseState,
        &mut PlayerWeaponRuntime,
    )>,
) {
    let reloaded = roster.is_changed();

    for (
        anchor_entity,
        anchor,
        mut transform,
        mount,
        pending,
        mut pose,
        mut runtime,
    ) in &mut anchors {
        if reloaded
            && let Some(mount) = mount
            && let Some(weapon) = roster.weapon(&mount.profile)
        {
            runtime.weapon = weapon.clone();
        }
        let weapon = &runtime.weapon;
        let profile_id = weapon.id.clone();

        *transform = anchor_transform(&roster);
        if reloaded {
            *pose = WeaponPoseState::default();
        }

        // Despawn + clear mount/pending on weapon change or reload.
        if let Some(mount) = mount {
            if !reloaded && mount.profile == profile_id {
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
                    resolved: true,
                });
                commands.entity(anchor_entity).remove::<PendingWeaponLoad>();
            }
            // Still loading: keep pending.
            continue;
        }

        // Start a new load, keeping the handle alive in a pending component.
        let handle = asset_server.load(&weapon.model);
        commands.entity(anchor_entity).insert(PendingWeaponLoad {
            handle,
            profile_id: weapon.id.clone(),
            failed: false,
        });
    }
}

/// Sockets are now driven purely by weapon configuration in `weapon.ron`.
/// This system ensures all mounts are marked resolved without traversing glTF nodes.
pub fn resolve_weapon_sockets(mut mounts: Query<&mut WeaponMount>) {
    for mut mount in &mut mounts {
        mount.resolved = true;
    }
}
