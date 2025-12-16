use bevy::prelude::*;
use shared::AppState;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Home), setup_home)
           .add_systems(OnExit(AppState::Home), cleanup_home);
    }
}

fn setup_home(_commands: Commands) {
    info!("Setting up Home Screen");
    // TODO: Add home screen UI
}

fn cleanup_home(_commands: Commands) {
    info!("Cleaning up Home Screen");
}
