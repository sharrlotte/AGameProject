pub mod component;
pub mod def;
pub mod generator;
pub mod hud;
pub mod lib;
pub mod system;

pub use component::*;

use bevy::prelude::*;

use crate::config::ConfigState;
use crate::config::loader::RonAssetLoader;
use crate::content::body::mutation::apply_body_mutations;
use crate::resource::SceneState;
use crate::save::world::WorldContext;
use crate::simulation::SimulationSet;

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<QuestCatalogDef>()
            .init_asset::<QuestGenConfig>()
            .register_asset_loader(RonAssetLoader::<QuestCatalogDef>::new(&["ron"]))
            .register_asset_loader(RonAssetLoader::<QuestGenConfig>::new(&["ron"]))
            .init_resource::<QuestCatalogValue>()
            .init_resource::<QuestGenConfigValue>()
            .init_resource::<QuestTracker>()
            .init_resource::<QuestOfferQueue>()
            .init_resource::<VisibleOffers>()
            .init_resource::<QuestAssetsReady>()
            .add_message::<QuestCompleted>()
            .add_systems(Startup, system::load_quest_handles)
            .add_systems(
                Update,
                (
                    system::load_initial_assets,
                    system::catalog_bridge,
                    system::generation_config_bridge,
                )
                    .run_if(in_state(ConfigState::Ready))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(OnEnter(SceneState::Game), hud::setup_quest_hud)
            .add_systems(
                Update,
                (
                    system::sync_offer_visibility,
                    hud::update_quest_tracker.run_if(resource_changed::<QuestTracker>),
                    hud::sync_offer_cards,
                    hud::sync_offer_dialog,
                )
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                FixedPostUpdate,
                (
                    system::activate_quests,
                    system::update_quest_objectives.after(apply_body_mutations),
                    system::generate_from_discovery,
                )
                    .run_if(in_state(SceneState::Game))
                    .run_if(save_slot_only)
                    .in_set(SimulationSet::Death),
            )
            .add_systems(
                FixedPostUpdate,
                system::respawn_player_on_defeat
                    .after(apply_body_mutations)
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Death),
            );
    }
}

pub fn save_slot_only(context: Res<WorldContext>) -> bool {
    context.is_save_slot()
}
