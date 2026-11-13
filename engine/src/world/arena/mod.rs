use bevy::prelude::*;
use crate::config::AppRonConfigExt;
use crate::common::{GameState, GameScene, SimUpdateSet};

use crate::config::ArenaConfig;

pub mod state;
pub mod system;

pub use state::{ArenaGround, ArenaObstacle, ArenaStaticEntity};
pub use system::spawn_arena_physics;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<ArenaConfig>()
            .add_systems(
                OnExit(GameState::Loading),
                system::spawn_arena_physics.run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                OnEnter(GameScene::Arena),
                system::spawn_arena_physics.run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                system::hot_reload_arena
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameScene::Arena))
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}

