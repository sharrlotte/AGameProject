use bevy::app::AnimationSystems;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;
use data::{AppRonConfigExt, GameState, SimUpdateSet};

use crate::config::VisualsConfig;

mod bind;
mod idle;
pub mod blend;
pub mod catalog;
pub mod director;
pub mod facing;
pub mod graph;
pub mod ik;
pub mod rig;
pub mod weapon;

pub use catalog::AnimCatalog;
pub use director::drive_player_animation;
pub use facing::{PlayerFacing, sync_player_facing};
pub use rig::PlayerRig;
pub use weapon::WeaponCatalog;

/// Marker on the visual root entity spawned as a direct child of `Survivor`.
#[derive(Component)]
pub struct PlayerVisualRoot;

#[derive(Component)]
pub struct PlayerModelChild;

#[derive(Component)]
pub(crate) struct PlayerModelSpawned;

/// Cached player glTF scene, layered graph, and locomotion clips.
#[derive(Resource, Default)]
pub struct PlayerAnimAssets {
    pub(crate) loaded_path: String,
    pub(crate) gltf: Option<Handle<Gltf>>,
    pub(crate) scene: Option<Handle<WorldAsset>>,
    pub(crate) graph: Option<Handle<AnimationGraph>>,
    pub(crate) nodes: Option<graph::GraphNodes>,
    pub(crate) locomotion: Option<director::LocomotionClips>,
}

impl PlayerAnimAssets {
    pub(crate) fn clear_built(&mut self) {
        self.scene = None;
        self.graph = None;
        self.nodes = None;
        self.locomotion = None;
    }
}

/// Post-update animation stages, all before transform propagation.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAnimSet {
    Drive,
    Weapon,
    Ik,
    Facing,
}

/// Orders the animation stages around Bevy's animation evaluation and transform propagation.
fn configure_anim_sets(app: &mut App) {
    app.configure_sets(PostUpdate, PlayerAnimSet::Drive.before(AnimationSystems))
        .configure_sets(PostUpdate, PlayerAnimSet::Weapon.after(AnimationSystems))
        .configure_sets(PostUpdate, PlayerAnimSet::Ik.after(PlayerAnimSet::Weapon))
        .configure_sets(
            PostUpdate,
            PlayerAnimSet::Facing
                .after(PlayerAnimSet::Ik)
                .before(TransformSystems::Propagate),
        );
}

/// Loads, binds, hot-reloads, and drives the layered player animation stack.
pub struct PlayerAnimPlugin;

impl Plugin for PlayerAnimPlugin {
    fn build(&self, app: &mut App) {
        configure_anim_sets(app);
        app.register_ron_config::<AnimCatalog>()
            .register_ron_config::<VisualsConfig>()
            .register_ron_config::<weapon::WeaponCatalog>()
            .init_resource::<PlayerAnimAssets>()
            .add_systems(OnExit(GameState::Loading), bind::start_loading_player_model)
            .add_systems(
                Update,
                (
                    facing::init_player_facing,
                    weapon::init_weapon_anchors,
                    bind::sync_player_visuals_hot_reload,
                    bind::sync_animation_catalog_hot_reload,
                    bind::prepare_player_model_assets,
                    bind::attach_player_model_scene,
                    bind::disable_player_backface_culling,
                    bind::bind_player_animation_players,
                    weapon::sync_weapon_mounts,
                    weapon::resolve_weapon_sockets,
                )
                    .chain()
                    .after(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                PostUpdate,
                (
                    weapon::update_weapon_kick,
                    weapon::update_weapon_swap,
                    weapon::pose_player_weapon,
                )
                    .chain()
                    .in_set(PlayerAnimSet::Weapon)
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::Playing | GameState::GameOver)
                    }),
            )
            .add_systems(
                PostUpdate,
                (
                    director::drive_player_animation.in_set(PlayerAnimSet::Drive),
                    ik::solve_player_arms.in_set(PlayerAnimSet::Ik),
                    idle::relax_idle_pose.in_set(PlayerAnimSet::Facing),
                    sync_player_facing.in_set(PlayerAnimSet::Facing),
                    #[cfg(feature = "dev-ik-gizmo")]
                    ik::draw_ik_gizmo.in_set(PlayerAnimSet::Ik),
                )
                    .run_if(|state: Res<State<GameState>>| {
                        matches!(state.get(), GameState::Playing | GameState::GameOver)
                    }),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Order(Vec<&'static str>);

    #[test]
    fn animation_stages_run_in_layer_order() {
        let mut app = App::new();
        configure_anim_sets(&mut app);
        app.init_resource::<Order>().add_systems(
            PostUpdate,
            (
                mark_drive.in_set(PlayerAnimSet::Drive),
                mark_anim.in_set(AnimationSystems),
                mark_weapon.in_set(PlayerAnimSet::Weapon),
                mark_ik.in_set(PlayerAnimSet::Ik),
                mark_facing.in_set(PlayerAnimSet::Facing),
                mark_propagate.in_set(TransformSystems::Propagate),
            ),
        );

        app.update();

        assert_eq!(
            app.world().resource::<Order>().0,
            ["drive", "anim", "weapon", "ik", "facing", "propagate"]
        );
    }

    fn mark_drive(mut order: ResMut<Order>) {
        order.0.push("drive");
    }

    fn mark_anim(mut order: ResMut<Order>) {
        order.0.push("anim");
    }

    fn mark_weapon(mut order: ResMut<Order>) {
        order.0.push("weapon");
    }

    fn mark_ik(mut order: ResMut<Order>) {
        order.0.push("ik");
    }

    fn mark_facing(mut order: ResMut<Order>) {
        order.0.push("facing");
    }

    fn mark_propagate(mut order: ResMut<Order>) {
        order.0.push("propagate");
    }
}
