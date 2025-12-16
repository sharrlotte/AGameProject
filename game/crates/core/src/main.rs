use bevy::prelude::*;
use bevy::window::WindowPlugin;
use shared::AppState;
use ui::{HomePlugin, GamePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AGameProject".to_string(),
                fullsize_content_view: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_state::<AppState>()
        .add_plugins((HomePlugin, GamePlugin))
        .add_systems(Startup, camera_setup)
        .run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
