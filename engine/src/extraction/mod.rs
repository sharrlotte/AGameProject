use bevy::prelude::*;
use crate::config::AppRonConfigExt;
use crate::common::{GameState, GameScene, SimPostPhysicsSet, SimUpdateSet};

pub mod flow;
pub mod pickup;
pub mod zone;

pub use crate::config::ExtractionConfig;
pub use pickup::ScrapPickup;
pub use zone::ExtractionZone;

pub struct ExtractionPlugin;

impl Plugin for ExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<ExtractionConfig>()
            .add_systems(
                OnEnter(GameState::Playing),
                zone::spawn_extraction_zone.run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                OnEnter(GameScene::Arena),
                zone::spawn_extraction_zone.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    zone::sync_extraction_hot_reload,
                    pickup::collect_scrap_pickups,
                    zone::update_extraction_zone,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                PostUpdate,
                pickup::spawn_scrap_on_enemy_killed
                    .in_set(SimPostPhysicsSet)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            );
    }
}

pub struct RunFlowPlugin;

impl Plugin for RunFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), flow::reset_run_stats)
            .add_systems(
                OnEnter(GameScene::Arena),
                flow::reset_run_stats.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::GameOver), flow::cleanup_run_entities)
            .add_systems(OnEnter(GameState::Upgrades), flow::cleanup_run_entities)
            .add_systems(
                Update,
                flow::tick_run_timer
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    flow::accumulate_kills_on_event,
                    flow::accumulate_scrap_on_event,
                    flow::handle_player_died_event,
                    flow::handle_extraction_completed_event,
                )
                    .after(SimPostPhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                flow::handle_navigation_requests.run_if(|state: Res<State<GameState>>| {
                    matches!(state.get(), GameState::GameOver | GameState::Upgrades)
                }),
            );
    }
}

#[cfg(test)]
mod tests;

