use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::common::{GROUP_WORLD, Obstacle};
use crate::world::Survivor;
use super::map::{
    ActiveInteriors, ColliderMesh, EnemySpawnerMarker, InteriorVolume, PlayerSpawnMarker,
    RoofOcclusion,
};

/// Tag to prevent re-processing nodes across multiple frames.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MapNodeProcessed;

/// Scans spawned glTF scene nodes and classifies them based on name prefixes.
pub fn inspect_map_scene_nodes(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    nodes: Query<
        (
            Entity,
            &Name,
            Option<&Mesh3d>,
            Option<&Transform>,
        ),
        Without<MapNodeProcessed>,
    >,
) {
    for (entity, name, mesh_handle, _transform) in &nodes {
        commands.entity(entity).insert(MapNodeProcessed);
        let name_str = name.as_str();

        if name_str.starts_with("Collider_") {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds.insert((
                ColliderMesh,
                Obstacle,
                RigidBody::Fixed,
                CollisionGroups::new(GROUP_WORLD, Group::ALL),
                Visibility::Hidden,
            ));

            if let Some(handle) = mesh_handle {
                if let Some(mesh) = meshes.get(handle.id()) {
                    if let Some(collider) = Collider::from_bevy_mesh(
                        mesh,
                        &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
                    ) {
                        entity_cmds.insert(collider);
                    }
                }
            }
        } else if name_str == "Spawn_Player" {
            commands.entity(entity).insert(PlayerSpawnMarker);
        } else if let Some(sub) = name_str.strip_prefix("Spawn_Enemy") {
            let kind_filter = sub.strip_prefix('_').map(|s| s.to_string());
            commands.entity(entity).insert(EnemySpawnerMarker { kind_filter });
        } else if let Some(id) = name_str.strip_prefix("Trigger_Interior_") {
            commands.entity(entity).insert((
                InteriorVolume {
                    id: id.to_string(),
                },
                Sensor,
                CollisionGroups::new(GROUP_WORLD, Group::ALL),
            ));
        } else if let Some(id) = name_str
            .strip_prefix("Roof_")
            .or_else(|| name_str.strip_prefix("UpperWall_"))
        {
            commands.entity(entity).insert(RoofOcclusion {
                interior_id: id.to_string(),
            });
        }
    }
}

/// Generates colliders for any ColliderMesh entities whose mesh finished loading asynchronously.
pub fn generate_pending_mesh_colliders(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    pending: Query<(Entity, &Mesh3d), (With<ColliderMesh>, Without<Collider>)>,
) {
    for (entity, handle) in &pending {
        if let Some(mesh) = meshes.get(handle.id()) {
            if let Some(collider) = Collider::from_bevy_mesh(
                mesh,
                &ComputedColliderShape::TriMesh(TriMeshFlags::default()),
            ) {
                commands.entity(entity).insert(collider);
            }
        }
    }
}

/// Detects survivor entering or exiting interior trigger volumes.
pub fn detect_survivor_interiors(
    mut collision_events: MessageReader<CollisionEvent>,
    survivors: Query<(), With<Survivor>>,
    mut survivor_interiors: Query<&mut ActiveInteriors, With<Survivor>>,
    interiors: Query<&InteriorVolume>,
) {
    for event in collision_events.read() {
        let (e1, e2, is_started) = match *event {
            CollisionEvent::Started(e1, e2, _) => (e1, e2, true),
            CollisionEvent::Stopped(e1, e2, _) => (e1, e2, false),
        };

        let (survivor_entity, interior_entity) = if survivors.contains(e1) && interiors.contains(e2) {
            (e1, e2)
        } else if survivors.contains(e2) && interiors.contains(e1) {
            (e2, e1)
        } else {
            continue;
        };

        let Ok(mut active) = survivor_interiors.get_mut(survivor_entity) else {
            continue;
        };
        let Ok(interior) = interiors.get(interior_entity) else {
            continue;
        };

        if is_started {
            if !active.ids.contains(&interior.id) {
                active.ids.push(interior.id.clone());
            }
        } else {
            active.ids.retain(|id| id != &interior.id);
        }
    }
}

/// Applies roof cutaway visibility based on the survivor's active interior volume.
pub fn apply_roof_cutaways(
    survivors: Query<&ActiveInteriors, With<Survivor>>,
    mut roofs: Query<(&RoofOcclusion, &mut Visibility)>,
) {
    for (roof, mut vis) in &mut roofs {
        let is_inside = survivors.iter().any(|s| s.ids.contains(&roof.interior_id));
        if is_inside {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
        }
    }
}

/// Repositions the survivor to the authored `Spawn_Player` marker location when loaded.
pub fn reposition_player_to_spawn_marker(
    spawns: Query<&Transform, Added<PlayerSpawnMarker>>,
    mut player: Query<&mut Transform, (With<Survivor>, Without<PlayerSpawnMarker>)>,
) {
    for spawn_tf in &spawns {
        for mut p_tf in &mut player {
            p_tf.translation = spawn_tf.translation;
            p_tf.rotation = spawn_tf.rotation;
        }
    }
}
