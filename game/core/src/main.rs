mod components;
mod content;
mod math;
mod save;
mod ui;

use crate::components::death::DeathPlugin;
use crate::components::id_gen::IdGen;
use crate::content::block::registry::{BlockRegistry, register_default_blocks};
use crate::content::bullet::BulletPlugin;
use crate::content::ship::ShipPlugin;
use crate::content::ship::base::ShipData;
use crate::content::turret::TurretPlugin;
use crate::save::load_saved_ships;

use bevy::input::common_conditions::input_toggle_active;
use bevy::post_process::bloom::BloomCompositeMode;
use bevy::window::{PresentMode, WindowPlugin};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy_rapier2d::prelude::*;
use ui::{GamePlugin, HomePlugin, PausePlugin, ShipEditorPlugin};

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
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
    .init_resource::<IdGen>()
    .init_state::<AppState>()
    .add_systems(PreStartup, register_default_blocks)
    .add_systems(Startup, (setup_camera, load_current_ship))
    .add_plugins((
        HomePlugin,
        GamePlugin,
        PausePlugin,
        ShipEditorPlugin,
        ShipPlugin,
        TurretPlugin,
        BulletPlugin,
        DeathPlugin,
    ));

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
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.3,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 1.0,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
        DebandDither::Enabled,
    ));
}

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum AppState {
    #[default]
    Home,
    Game,
    ShipEditor,
    Pause,
}

#[derive(Resource, Default)]
pub struct AppVar {
    pub current_ship: Option<ShipData>,
}

pub const GRID_SIZE: f32 = 64.0;

fn load_current_ship(mut state: ResMut<AppVar>) {
    let ships = load_saved_ships();

    if let Some(ship) = ships.first() {
        state.current_ship = Some(ship.clone());
    }
}
