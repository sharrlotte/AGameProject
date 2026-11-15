use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::common::{GROUP_WORLD, Obstacle};
use crate::config::PlayerConfig;
use crate::world::Survivor;
use super::map::{
    ActiveInteriors, ColliderMesh, EnemySpawnerMarker, InteriorVolume, PlayerSpawnMarker,
    RoofOcclusion,
};

/// Tag to prevent re-processing nodes across multiple frames.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MapNodeProcessed;

/// Resolves a mesh handle from an entity or its descendants, because glTF mesh primitives spawn on child entities.
fn resolve_mesh_handle(
    entity: Entity,
    meshes: &Query<&Mesh3d>,
    children: &Query<&Children>,
) -> Option<Handle<Mesh>> {
    if let Ok(mesh) = meshes.get(entity) {
        return Some(mesh.0.clone());
    }

    let mut stack = vec![entity];
    while let Some(current) = stack.pop() {
        let Ok(child_list) = children.get(current) else {
            continue;
        };
        for child in child_list.iter() {
            if let Ok(mesh) = meshes.get(child) {
                return Some(mesh.0.clone());
            }
            stack.push(child);
        }
    }
    None
}

/// Shape used when converting a map node's mesh into a collider.
#[derive(Clone, Copy)]
enum ColliderShape {
    Surface,
    Solid,
}

impl ColliderShape {
    fn computed(self) -> ComputedColliderShape {
        match self {
            Self::Surface => ComputedColliderShape::TriMesh(TriMeshFlags::default()),
            Self::Solid => ComputedColliderShape::ConvexHull,
        }
    }
}

/// Builds a collider from the node's resolved mesh when the asset is loaded.
fn mesh_collider(
    entity: Entity,
    shape: ColliderShape,
    meshes: &Assets<Mesh>,
    mesh_query: &Query<&Mesh3d>,
    children: &Query<&Children>,
) -> Option<Collider> {
    let handle = resolve_mesh_handle(entity, mesh_query, children)?;
    let mesh = meshes.get(&handle)?;
    Collider::from_bevy_mesh(mesh, &shape.computed())
}

/// Scans spawned glTF scene nodes and classifies them based on name prefixes.
#[allow(clippy::type_complexity)]
pub fn inspect_map_scene_nodes(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
    children: Query<&Children>,
    nodes: Query<(Entity, &Name), Without<MapNodeProcessed>>,
) {
    for (entity, name) in &nodes {
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

            if let Some(collider) =
                mesh_collider(entity, ColliderShape::Surface, &meshes, &mesh_query, &children)
            {
                entity_cmds.insert(collider);
            }
        } else if name_str.starts_with("Occluder_") {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds.insert((
                ColliderMesh,
                Obstacle,
                RigidBody::Fixed,
                CollisionGroups::new(GROUP_WORLD, Group::ALL),
            ));

            if let Some(collider) =
                mesh_collider(entity, ColliderShape::Surface, &meshes, &mesh_query, &children)
            {
                entity_cmds.insert(collider);
            }
        } else if name_str == "Spawn_Player" {
            commands.entity(entity).insert(PlayerSpawnMarker);
        } else if let Some(sub) = name_str.strip_prefix("Spawn_Enemy") {
            let kind_filter = sub.strip_prefix('_').map(|s| s.to_string());
            commands.entity(entity).insert(EnemySpawnerMarker { kind_filter });
        } else if let Some(id) = name_str.strip_prefix("Trigger_Interior_") {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds.insert((
                InteriorVolume {
                    id: id.to_string(),
                },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(GROUP_WORLD, Group::ALL),
                Visibility::Hidden,
            ));

            if let Some(collider) =
                mesh_collider(entity, ColliderShape::Solid, &meshes, &mesh_query, &children)
            {
                entity_cmds.insert(collider);
            }
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

/// Generates colliders for any ColliderMesh or InteriorVolume entity whose mesh finished loading asynchronously.
#[allow(clippy::type_complexity)]
pub fn generate_pending_mesh_colliders(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
    children: Query<&Children>,
    pending: Query<
        (Entity, Option<&InteriorVolume>),
        (
            Or<(With<ColliderMesh>, With<InteriorVolume>)>,
            Without<Collider>,
        ),
    >,
) {
    for (entity, interior) in &pending {
        let shape = if interior.is_some() {
            ColliderShape::Solid
        } else {
            ColliderShape::Surface
        };
        if let Some(collider) = mesh_collider(entity, shape, &meshes, &mesh_query, &children) {
            commands.entity(entity).insert(collider);
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
    player_cfg: Res<PlayerConfig>,
    spawns: Query<&Transform, Added<PlayerSpawnMarker>>,
    mut player: Query<&mut Transform, (With<Survivor>, Without<PlayerSpawnMarker>)>,
) {
    for spawn_tf in &spawns {
        for mut p_tf in &mut player {
            p_tf.translation = Vec3::new(
                spawn_tf.translation.x,
                player_cfg.aim_height,
                spawn_tf.translation.z,
            );
            p_tf.rotation = spawn_tf.rotation;
        }
    }
}
