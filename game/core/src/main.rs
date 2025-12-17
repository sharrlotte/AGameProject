mod content;
mod ui;
mod save;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use content::structures::registry::{BlockRegistry, register_default_blocks};
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
    .init_resource::<BlockRegistry>()
    .init_state::<AppState>()
    .add_systems(PreStartup, register_default_blocks)
    .add_systems(Startup, setup_camera)
    .add_plugins((HomePlugin, GamePlugin, ShipEditorPlugin));

    #[cfg(debug_assertions)]
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)));

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Home,
    Game,
    ShipEditor,
}
