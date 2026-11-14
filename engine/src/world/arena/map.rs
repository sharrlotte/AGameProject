use bevy::prelude::*;

/// Root marker on the spawned map glTF scene entity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MapRoot;

/// Marker on player spawn position/facing extracted from `Spawn_Player`.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerSpawnMarker;

/// Marker on enemy spawner positions extracted from `Spawn_Enemy` or `Spawn_Enemy_{Kind}`.
#[derive(Component, Debug, Clone, Default)]
pub struct EnemySpawnerMarker {
    pub kind_filter: Option<String>,
}

/// Sensor volume identifying a building interior trigger (`Trigger_Interior_{id}`).
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct InteriorVolume {
    pub id: String,
}

/// Marker on roof or upper-wall visual meshes linked to an interior (`Roof_{id}` or `UpperWall_{id}`).
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct RoofOcclusion {
    pub interior_id: String,
}

/// Marker on extracted collision nodes (`Collider_*`).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ColliderMesh;

/// Tracks which interior volume(s) the survivor is currently inside.
#[derive(Component, Debug, Clone, Default)]
pub struct ActiveInteriors {
    pub ids: Vec<String>,
}
