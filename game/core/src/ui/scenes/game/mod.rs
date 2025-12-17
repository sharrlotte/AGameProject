mod ship;

use bevy::prelude::*;

use crate::{AppState, content::ships::features::ShipFeature, ui::scenes::game::ship::ShipPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_game)
            .add_systems(OnExit(AppState::Game), cleanup_game)
            .add_plugins(ShipFeature)
            .add_plugins(ShipPlugin);
    }
}

fn setup_game(_commands: Commands) {
    info!("Setting up Game Scene");
}

fn cleanup_game(_commands: Commands) {
    info!("Cleaning up Game Scene");
}
