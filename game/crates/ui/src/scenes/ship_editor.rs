use bevy::prelude::*;
use shared::AppState;

pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::ShipEditor), setup_ship_editor)
           .add_systems(OnExit(AppState::ShipEditor), cleanup_ship_editor);
    }
}

fn setup_ship_editor(mut commands: Commands) {
    info!("Setting up Ship Editor Scene");
    // TODO: Add ship editor setup
}

fn cleanup_ship_editor(mut commands: Commands) {
    info!("Cleaning up Ship Editor Scene");
}
