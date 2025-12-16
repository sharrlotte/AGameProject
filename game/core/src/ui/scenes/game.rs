use bevy::prelude::*;
use crate::content::AppState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_game)
           .add_systems(OnExit(AppState::Game), cleanup_game);
    }
}

fn setup_game(_commands: Commands) {
    info!("Setting up Game Scene");
    // TODO: Add game setup
}

fn cleanup_game(_commands: Commands) {
    info!("Cleaning up Game Scene");
}
