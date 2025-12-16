use bevy::prelude::*;
use bevy::window::WindowPlugin;
use shared::AppState;
use shared::structure::registry::{BlockRegistry, register_default_blocks};
use ui::{GamePlugin, HomePlugin, ShipEditorPlugin};

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
    .init_resource::<BlockRegistry>()
    .add_systems(Startup, (camera_setup, register_default_blocks))
    .add_plugins((HomePlugin, GamePlugin, ShipEditorPlugin));

    #[cfg(debug_assertions)]
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
