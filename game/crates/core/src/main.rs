use bevy::prelude::*;
use bevy::window::WindowPlugin;
use shared::AppState;
use ui::{GamePlugin, HomePlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AGameProject".to_string(),
            // mode: WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .init_state::<AppState>()
    .add_plugins((HomePlugin, GamePlugin))
    .add_systems(Startup, camera_setup);

    #[cfg(debug_assertions)]
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
