use bevy::app::AnimationSystems;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;
use data::{AppRonConfigExt, GameState, SimUpdateSet};

use crate::config::VisualsConfig;

mod bind;
pub mod blend;
pub mod catalog;
pub mod director;
pub mod facing;
pub mod graph;
pub mod ik;
pub mod rig;
pub mod stance;
pub mod twist;

pub use catalog::AnimCatalog;
pub use director::drive_player_animation;
pub use facing::{MoveZone, PlayerFacing, sync_player_facing};
pub use rig::PlayerRig;

/// Marker on the visual root entity spawned as a direct child of `Survivor`.
#[derive(Component)]
pub struct PlayerVisualRoot;

#[derive(Component)]
pub struct PlayerModelChild;

#[derive(Component)]
pub(crate) struct PlayerModelSpawned;

/// Cached player glTF scene, layered graph, and clip nodes.
#[derive(Resource, Default)]
pub struct PlayerAnimAssets {
    pub(crate) loaded_path: String,
    pub(crate) gltf: Option<Handle<Gltf>>,
    pub(crate) scene: Option<Handle<WorldAsset>>,
    pub(crate) graph: Option<Handle<AnimationGraph>>,
    pub(crate) nodes: Option<graph::GraphNodes>,
}

impl PlayerAnimAssets {
    pub(crate) fn clear_built(&mut self) {
        self.scene = None;
        self.graph = None;
        self.nodes = None;
    }
}

/// Runs after animation evaluation and spine twist, before transform propagation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerIkSet;

/// Loads, binds, hot-reloads, and drives the layered player animation stack.
pub struct PlayerAnimPlugin;

impl Plugin for PlayerAnimPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<AnimCatalog>()
            .register_ron_config::<VisualsConfig>()
            .init_resource::<PlayerAnimAssets>()
            .configure_sets(
                PostUpdate,
                PlayerIkSet
                    .after(twist::apply_upper_body_twist)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(OnExit(GameState::Loading), bind::start_loading_player_model)
            .add_systems(
                Update,
                (
                    facing::init_player_facing,
                    bind::sync_player_visuals_hot_reload,
                    bind::sync_animation_catalog_hot_reload,
                    bind::prepare_player_model_assets,
                    bind::attach_player_model_scene,
                    bind::disable_player_backface_culling,
                    bind::bind_player_animation_players,
                )
                    .chain()
                    .after(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                (sync_player_facing, drive_player_animation)
                    .chain()
                    .after(bind::bind_player_animation_players)
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::Playing | GameState::GameOver)
                    }),
            )
            .add_systems(
                PostUpdate,
                (
                    twist::apply_upper_body_twist.after(AnimationSystems),
                    ik::apply_support_hand_ik.in_set(PlayerIkSet),
                    #[cfg(feature = "dev-ik-gizmo")]
                    ik::draw_ik_gizmo.in_set(PlayerIkSet),
                )
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::Playing | GameState::GameOver)
                    }),
            );
    }
}
