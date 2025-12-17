mod content;
mod save;
mod ui;

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_rapier2d::prelude::*;
use content::structures::registry::{BlockRegistry, register_default_blocks};
use ui::{GamePlugin, HomePlugin, ShipEditorPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::content::ships::features::ShipFeature;
use crate::{content::structures::ship::SavedShip, save::load_saved_ships};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AGameProject".to_string(),
            present_mode: PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugins(RapierDebugRenderPlugin::default())
    .init_resource::<BlockRegistry>()
    .init_resource::<AppVar>()
    .init_state::<AppState>()
    .add_systems(PreStartup, register_default_blocks)
    .add_systems(Startup, (setup_camera, load_current_ship))
    .add_plugins((HomePlugin, GamePlugin, ShipEditorPlugin, ShipFeature));

    #[cfg(debug_assertions)]
    {
        app.add_plugins(EguiPlugin::default());
        app.add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
        );
    }

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

#[derive(Resource, Default)]
pub struct AppVar {
    pub current_ship: Option<SavedShip>,
}

pub const GRID_SIZE: f32 = 64.0;

fn load_current_ship(mut state: ResMut<AppVar>) {
    let ships = load_saved_ships();

    if let Some(ship) = ships.first() {
        state.current_ship = Some(ship.clone());
    }
}
