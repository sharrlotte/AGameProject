use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AGameProject".to_string(),
                resolution: WindowResolution::new(700, 800),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, camera_setup)
        .run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
