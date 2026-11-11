use bevy::prelude::*;
use data::{GameState, SimPostPhysicsSet, SimUpdateSet};

pub mod arena;
pub mod combat;
pub mod enemy;
pub mod extract;
pub mod material;
pub mod player;

pub use enemy::VisualBody;
pub use extract::ExtractionPadVisual;
pub use material::SharedVisualMaterials;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedVisualMaterials>()
            .add_systems(OnExit(GameState::Loading), arena::setup_lighting)
            .add_systems(
                Update,
                (
                    arena::sync_lighting_hot_reload,
                    arena::attach_arena_visuals,
                    player::attach_player_visuals,
                    combat::attach_projectile_visuals,
                    enemy::attach_enemy_visuals,
                    extract::attach_pickup_visuals,
                    extract::attach_extraction_visuals,
                    extract::animate_scrap_pickups,
                    extract::update_extraction_pad_visual,
                )
                    .chain()
                    .after(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                PostUpdate,
                enemy::sync_visual_facing
                    .after(SimPostPhysicsSet)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
