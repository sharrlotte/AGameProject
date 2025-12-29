mod common;
mod constant;
mod content;
mod resource;
mod save;
mod scene;
mod system;
mod ui;
mod utils;
mod world;

use crate::common::damage::DamagePlugin;
use crate::common::death::DeathPlugin;
use crate::common::lifetime::LifetimePlugin;
use crate::constant::AUDIO_SCALE;
use crate::content::plugin::ContentPlugin;
use crate::resource::{GameVars, PauseState, SceneState};
use crate::save::io::SavePlugin;
use crate::scene::editor::ShipEditorPlugin;
use crate::scene::game::GameScenePlugin;
use crate::scene::home::HomeScenePlugin;
use crate::scene::pause::PauseScenePlugin;
use crate::scene::setting::SettingScenePlugin;
use crate::system::{clean_state_item, load_current_ship, setup_camera};
use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::IntegrationParameters;

#[cfg(debug_assertions)]
use bevy::input::common_conditions::input_toggle_active;

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "AGameProject".to_string(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AudioPlugin {
                default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                ..default()
            }),
    )
    .add_plugins(
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0)
            .with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    integration_parameters: IntegrationParameters { ..default() },
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
    // .add_plugins(RapierDebugRenderPlugin::default())
    .init_resource::<GameVars>()
    .init_state::<SceneState>()
    .init_state::<PauseState>()
    .add_systems(Startup, (setup_camera, load_current_ship))
    .add_plugins((
        FrameTimeDiagnosticsPlugin::new(100),
        EntityCountDiagnosticsPlugin::new(100),
        GameScenePlugin,
        HomeScenePlugin,
        PauseScenePlugin,
        ShipEditorPlugin,
        DeathPlugin,
        SavePlugin,
        DamagePlugin,
        HanabiPlugin,
        ContentPlugin,
        SettingScenePlugin,
        LifetimePlugin,
    ))
    .insert_resource(Time::<Fixed>::from_hz(60.0))
    .add_systems(PreUpdate, clean_state_item);

    #[cfg(debug_assertions)]
    {
        app.add_plugins(EguiPlugin::default()).add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
        );
    }

    app.run();
}
