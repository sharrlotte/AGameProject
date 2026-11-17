mod constant;
mod config;
mod content;
mod mission;
mod resource;
mod save;
mod scene;
mod simulation;
mod system;

use crate::constant::AUDIO_SCALE;
use crate::config::ConfigPlugin;
use crate::content::plugin::ContentPlugin;
use crate::mission::MissionPlugin;
use crate::simulation::SimulationSet;
use crate::resource::{CameraTarget, ControlledShip, DeployPayload, PauseState, SceneState};
use crate::save::io::SavePlugin;
use crate::scene::editor::ShipEditorPlugin;
use crate::scene::game::GameScenePlugin;
use crate::scene::home::HomeScenePlugin;
use crate::scene::notice::NoticePlugin;
use crate::scene::pause::PauseScenePlugin;
use crate::scene::setting::SettingScenePlugin;
use crate::system::{load_current_ship, setup_camera};
#[cfg(debug_assertions)]
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::IntegrationParameters;

#[cfg(feature = "dev")]
use bevy::remote::RemotePlugin;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;

#[cfg(all(debug_assertions, feature = "dev"))]
use bevy::input::common_conditions::input_toggle_active;

#[cfg(all(debug_assertions, feature = "dev"))]
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
                        simulation_mode: SimulationMode::Full,
                        num_threads: None,
                    },
                    broad_phase_optimization_strategy: BroadPhaseOptimizationStrategy::default(),
                },
            )
            .in_fixed_schedule(),
    )
    // .add_plugins(RapierDebugRenderPlugin::default())
    .init_resource::<DeployPayload>()
    .init_resource::<CameraTarget>()
    .init_resource::<ControlledShip>()
    .init_state::<SceneState>()
    .init_state::<PauseState>()
    .configure_sets(PreUpdate, SimulationSet::Input)
    .configure_sets(FixedFirst, SimulationSet::Stats)
    .configure_sets(FixedPreUpdate, SimulationSet::Stats)
    .configure_sets(
        FixedUpdate,
        (
            SimulationSet::Ai,
            SimulationSet::Control,
            SimulationSet::Weapons,
            SimulationSet::Cleanup,
        )
            .chain(),
    )
    .configure_sets(
        FixedPostUpdate,
        (SimulationSet::Damage, SimulationSet::Death).chain(),
    )
    .configure_sets(FixedLast, SimulationSet::Cleanup)
    .configure_sets(Update, (SimulationSet::Input, SimulationSet::Present).chain())
    .configure_sets(PostUpdate, SimulationSet::Present)
    .add_systems(Startup, (setup_camera, load_current_ship))
    .add_plugins((
        #[cfg(debug_assertions)]
        FrameTimeDiagnosticsPlugin::new(100),
        #[cfg(debug_assertions)]
        EntityCountDiagnosticsPlugin::new(100),
        ConfigPlugin,
        MissionPlugin,
        GameScenePlugin,
        HomeScenePlugin,
        PauseScenePlugin,
        ShipEditorPlugin,
        SavePlugin,
        HanabiPlugin,
        ContentPlugin,
        SettingScenePlugin,
        NoticePlugin,
    ))
    .insert_resource(Time::<Fixed>::from_hz(60.0));

    #[cfg(all(debug_assertions, feature = "dev"))]
    {
        app.add_plugins(EguiPlugin::default()).add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
        );
    }

    #[cfg(feature = "dev")]
    {
        app.add_plugins((RemotePlugin::default(), BrpExtrasPlugin));
    }

    app.run();
}
