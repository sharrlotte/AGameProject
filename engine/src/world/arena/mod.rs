use bevy::prelude::*;
use crate::config::AppRonConfigExt;
use crate::common::{GameState, GameScene, SimUpdateSet};

use crate::config::ArenaConfig;

pub mod loader;
pub mod map;
pub mod state;
pub mod system;

pub use loader::{
    apply_roof_cutaways, detect_survivor_interiors, generate_pending_mesh_colliders,
    inspect_map_scene_nodes, reposition_player_to_spawn_marker,
};
pub use map::{
    ActiveInteriors, ColliderMesh, EnemySpawnerMarker, InteriorVolume, MapRoot, PlayerSpawnMarker,
    RoofOcclusion,
};
pub use state::{ArenaGround, ArenaObstacle, ArenaStaticEntity};
pub use system::{spawn_arena_map, spawn_arena_physics};

#[cfg(test)]
mod tests;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<ArenaConfig>()
            .add_systems(
                OnExit(GameState::Loading),
                (system::spawn_arena_physics, system::spawn_arena_map)
                    .run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                OnEnter(GameScene::Arena),
                (system::spawn_arena_physics, system::spawn_arena_map)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                (
                    system::hot_reload_arena,
                    loader::inspect_map_scene_nodes,
                    loader::generate_pending_mesh_colliders,
                    loader::detect_survivor_interiors,
                    loader::apply_roof_cutaways,
                    loader::reposition_player_to_spawn_marker,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameScene::Arena))
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}

