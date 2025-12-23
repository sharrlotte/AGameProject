mod components;
mod constant;
mod content;
mod save;
mod ui;
mod utils;
mod world;

use crate::components::damage::DamagePlugin;
use crate::components::death::DeathPlugin;
use crate::content::block::BlockPlugin;
use crate::content::block::registry::BlockRegistry;
use crate::content::bullet::BulletPlugin;
use crate::content::ship::ShipPlugin;
use crate::content::turret::TurretPlugin;
use crate::save::SavePlugin;
use crate::save::base::SchematicData;
use crate::save::io::load_saved_schematics;
use crate::world::physic::PhysicPlugin;

use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
#[cfg(debug_assertions)]
use bevy::input::common_conditions::input_toggle_active;

use bevy::post_process::bloom::BloomCompositeMode;
use bevy::window::{PresentMode, WindowPlugin};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::IntegrationParameters;
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
    .add_plugins(
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0)
            .with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    integration_parameters: IntegrationParameters::default(),
                    rapier_configuration: RapierConfiguration {
                        gravity: Vec2::ZERO,
                        physics_pipeline_active: true,
                        scaled_shape_subdivision: 10,
                        force_update_from_transform_changes: false,
                    },
                },
            )
            .in_fixed_schedule(),
    )
    .add_plugins(RapierDebugRenderPlugin::default())
    .init_resource::<BlockRegistry>()
    .init_resource::<GameState>()
    .init_state::<SceneState>()
    .init_state::<PauseState>()
    .add_systems(Startup, (setup_camera, load_current_ship))
    .add_plugins((
        FrameTimeDiagnosticsPlugin::new(1000),
        EntityCountDiagnosticsPlugin::new(1000),
        //
        HomePlugin,
        GamePlugin,
        PausePlugin,
        ShipEditorPlugin,
        ShipPlugin,
        TurretPlugin,
        BulletPlugin,
        DeathPlugin,
        SavePlugin,
        BlockPlugin,
        DamagePlugin,
        HanabiPlugin,
        PhysicPlugin,
    ))
    .insert_resource(Time::<Fixed>::from_hz(60.0));

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
pub enum SceneState {
    #[default]
    Home,
    Game,
    ShipEditor,
}

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum PauseState {
    #[default]
    Playing,
    Paused,
}

#[derive(Default, PartialEq, Clone)]
pub enum TurretControl {
    #[default]
    Auto,
    Manual,
    Hold,
}

impl TurretControl {
    pub fn next(&mut self) -> Self {
        match self {
            Self::Auto => Self::Manual,
            Self::Manual => Self::Hold,
            Self::Hold => Self::Auto,
        }
    }
}

#[derive(Resource, Default)]
pub struct GameState {
    pub current_ship: Option<SchematicData>,
    pub turret_control: TurretControl,
}

fn load_current_ship(mut state: ResMut<GameState>) {
    let ships = load_saved_schematics();

    if let Some(ship) = ships.first() {
        state.current_ship = Some(ship.clone());
    }
}
