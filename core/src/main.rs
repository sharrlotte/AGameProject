mod asset;
mod config;
mod constant;
mod content;
mod control;
mod mission;
mod resource;
mod save;
mod scene;
mod simulation;
mod system;
mod ui;

use crate::config::{ConfigPlugin, ConfigState, GameConfigValue};
use crate::constant::AUDIO_SCALE;
use crate::content::ContentPlugin;
use crate::mission::MissionPlugin;
use crate::resource::{
    DeployPayload, GameRng, PauseState, SceneState,
};
use crate::save::SavePlugin;
use crate::scene::editor::ShipEditorPlugin;
use crate::scene::game::GameScenePlugin;
use crate::scene::home::HomeScenePlugin;
use crate::scene::notice::NoticePlugin;
use crate::scene::pause::PauseScenePlugin;
use crate::scene::setting::SettingScenePlugin;
use crate::control::ControlPlugin;
use crate::simulation::SimulationSet;
use crate::system::{load_default_fleet, setup_camera};
use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::window::WindowLevel::AlwaysOnBottom;
use bevy::window::{MonitorSelection, WindowMode, WindowPlugin};
use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::IntegrationParameters;

#[cfg(feature = "dev")]
use {
    crate::asset::dev::AssetCheckPlugin, bevy::remote::RemotePlugin, bevy_brp_extras::BrpExtrasPlugin,
};

#[cfg(all(debug_assertions, feature = "dev"))]
use {
    bevy::input::common_conditions::input_toggle_active,
    bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin},
};

fn main() {
    let mut app = App::new();

    register_engine_plugins(&mut app);
    register_resources_and_states(&mut app);
    register_simulation_sets(&mut app);
    register_game_plugins(&mut app);
    register_debug_and_dev_plugins(&mut app);

    app.run();
}

fn register_engine_plugins(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "AGameProject".to_string(),
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    window_level: AlwaysOnBottom,
                    present_mode: PresentMode::AutoVsync,
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
                    integration_parameters: IntegrationParameters { 
                        normalized_max_linear_velocity: f32::MAX,
                        ..default() 
                    },
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
    );
}

fn register_resources_and_states(app: &mut App) {
    app.init_resource::<DeployPayload>()
        .init_resource::<GameRng>()
        // Gameplay systems assume config exists; a failed load still shows the
        // error screen instead of panicking on a missing resource.
        .init_resource::<GameConfigValue>()
        .init_state::<SceneState>()
        .init_state::<PauseState>()
        .insert_resource(Time::<Fixed>::from_hz(60.0));
}

fn register_simulation_sets(app: &mut App) {
    app.configure_sets(PreUpdate, SimulationSet::Input)
        .configure_sets(FixedFirst, SimulationSet::Stats)
        .configure_sets(FixedPreUpdate, SimulationSet::Stats)
        .configure_sets(
            FixedUpdate,
            (
                SimulationSet::Ai,
                SimulationSet::Control,
                SimulationSet::Weapons,
                SimulationSet::Cleanup,
                PhysicsSet::SyncBackend,
            )
                .chain(),
        )
        .configure_sets(
            FixedPostUpdate,
            (SimulationSet::Damage, SimulationSet::Death).chain(),
        )
        .configure_sets(FixedLast, SimulationSet::Cleanup)
        .configure_sets(
            Update,
            (SimulationSet::Input, SimulationSet::Present).chain(),
        )
        .configure_sets(PostUpdate, SimulationSet::Present);
}

fn register_game_plugins(app: &mut App) {
    app.add_systems(Startup, setup_camera)
        .add_systems(OnEnter(ConfigState::Ready), load_default_fleet)
        .add_plugins((
            ConfigPlugin,
            ControlPlugin,
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
            scene::widget::WidgetPlugin,
        ));
}

fn register_debug_and_dev_plugins(app: &mut App) {
    #[cfg(debug_assertions)]
    {
        use crate::system::debug_audit::DebugAuditPlugin;
        use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};

        app.add_plugins((
            FrameTimeDiagnosticsPlugin::new(100),
            EntityCountDiagnosticsPlugin::new(100),
            DebugAuditPlugin,
        ));

        #[cfg(feature = "dev")]
        {
            app.add_plugins(EguiPlugin::default()).add_plugins(
                WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
            );
        }
    }

    #[cfg(feature = "dev")]
    {
        app.add_plugins((
            RemotePlugin::default(),
            BrpExtrasPlugin,
            AssetCheckPlugin,
        ));
    }
}
