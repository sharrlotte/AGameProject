use bevy::prelude::*;
use data::{AppRonConfigExt, GameState, SimUpdateSet};

use crate::config::ArenaConfig;

pub mod state;
pub mod system;

pub use state::{ArenaGround, ArenaObstacle, ArenaStaticEntity};
pub use system::spawn_arena_physics;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<ArenaConfig>()
            .add_systems(OnExit(GameState::Loading), system::spawn_arena_physics)
            .add_systems(
                Update,
                system::hot_reload_arena
                    .in_set(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}
