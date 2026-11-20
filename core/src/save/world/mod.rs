pub mod component;
pub mod snapshot;
pub mod sqlite;
pub mod store;
pub mod system;

pub use component::*;

use bevy::prelude::*;

use crate::mission::BattlePhase;
use crate::resource::{BattleMode, SceneState};
use crate::save::world::system::{
    begin_loaded_game, flush_pending_exit_action, handle_continue_world, handle_create_slot,
    handle_load_slot, handle_quick_battle, handle_save_world_event, init_world_session,
    reset_auto_save_timer, reset_catalog_on_home, spawn_loaded_world, tick_auto_save,
};
use crate::simulation::SimulationSet;
use crate::ui::{Modal, ModalStack};

pub struct WorldSavePlugin;

impl Plugin for WorldSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadSlotEvent>()
            .add_message::<CreateSlotEvent>()
            .add_message::<StartQuickBattleEvent>()
            .add_message::<ContinueWorldEvent>()
            .init_resource::<WorldContext>()
            .init_resource::<EditorReturnTarget>()
            .init_resource::<PendingExitAction>()
            .init_resource::<BattleMode>()
            .add_systems(PreStartup, init_world_session)
            .add_systems(
                Update,
                begin_loaded_game
                    .run_if(in_state(SceneState::Loading))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                OnEnter(SceneState::Game),
                (reset_auto_save_timer, spawn_loaded_world).chain(),
            )
            .add_systems(OnEnter(SceneState::Home), reset_catalog_on_home)
            .add_systems(
                Update,
                (
                    handle_save_world_event
                        .run_if(on_message::<SaveWorldEvent>)
                        .before(flush_pending_exit_action),
                    flush_pending_exit_action,
                    handle_load_slot.run_if(on_message::<LoadSlotEvent>),
                    handle_create_slot.run_if(on_message::<CreateSlotEvent>),
                    handle_continue_world.run_if(on_message::<ContinueWorldEvent>),
                    handle_quick_battle.run_if(on_message::<StartQuickBattleEvent>),
                    tick_auto_save
                        .run_if(in_state(SceneState::Game))
                        .run_if(in_state(BattlePhase::Active))
                        .run_if(|stack: Res<ModalStack>| stack.top() != Some(Modal::Pause)),
                )
                    .in_set(SimulationSet::Present),
            );
    }
}
