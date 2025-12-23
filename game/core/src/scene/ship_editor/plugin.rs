use bevy::prelude::*;

use crate::{
    SceneState,
    scene::ship_editor::{
        message::{ShipSaveChanged, ShipSavesChanged},
        resource::EditorState,
        system::{
            cleanup_ship_editor, draw_grid, handle_block_deletion, handle_block_selection, handle_deploy_button, handle_deploy_button_click, handle_nav_buttons, handle_placement, handle_ship_loading, handle_ship_saving, handle_tab_switching, load_initial_ship, on_scroll_handler, process_input, refresh_ship_list, send_scroll_events, setup_editor_ui, update_block_properties_panel, update_placement_preview
        },
    },
};

pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShipSavesChanged>()
            .add_message::<ShipSaveChanged>()
            .init_resource::<EditorState>()
            .add_systems(
                OnEnter(SceneState::ShipEditor),
                (setup_editor_ui, load_initial_ship, refresh_ship_list).chain(),
            )
            .add_systems(OnExit(SceneState::ShipEditor), cleanup_ship_editor)
            .add_systems(
                Update,
                (
                    handle_block_selection,
                    handle_tab_switching,
                    handle_ship_loading,
                    process_input,
                    draw_grid,
                    update_placement_preview,
                    handle_placement,
                    handle_block_deletion,
                    update_block_properties_panel,
                    handle_deploy_button,
                    handle_deploy_button_click,
                    handle_nav_buttons,
                    handle_ship_saving.run_if(on_message::<ShipSaveChanged>),
                    refresh_ship_list.run_if(on_message::<ShipSavesChanged>),
                )
                    .chain()
                    .run_if(in_state(SceneState::ShipEditor)),
            )
            .add_systems(Update, send_scroll_events)
            .add_observer(on_scroll_handler);
    }
}
