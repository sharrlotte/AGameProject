use bevy::audio::AudioPlugin;
use bevy::diagnostic::EntityCountDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowPlugin};
use bevy_rapier3d::prelude::*;
use engine::combat::CombatPlugin;
use engine::common::CoreDataPlugin;
use engine::enemies::EnemiesPlugin;
use engine::extraction::{ExtractionPlugin, RunFlowPlugin, UpgradesPlugin};
use engine::presentation::AllPresentationPlugins;
use engine::world::{ArenaPlugin, PlayerPlugin};

#[cfg(feature = "dev")]
use bevy::input::common_conditions::input_toggle_active;
#[cfg(feature = "dev")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
#[cfg(feature = "dev")]
use engine::world::AnimTestPlugin;

#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;

/// Builds the configured Bevy application with all plugins, systems, and diagnostics.
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "AGameProject".to_string(),
                    present_mode: PresentMode::AutoVsync,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AudioPlugin::default()),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(bevy::remote::RemotePlugin::default()) // Enable BRP
    .add_plugins(BrpExtrasPlugin) // Enhanced BRP features
    .insert_resource(TimestepMode::Interpolated {
        dt: 1.0 / 60.0,
        time_scale: 1.0,
        substeps: 1,
    })
    .add_plugins((
        EntityCountDiagnosticsPlugin::new(100),
    ))
    .add_plugins((
        CoreDataPlugin,
        ArenaPlugin,
        PlayerPlugin,
        CombatPlugin,
        EnemiesPlugin,
        ExtractionPlugin,
        RunFlowPlugin,
        UpgradesPlugin,
        AllPresentationPlugins,
    ))
    .add_systems(Startup, configure_rapier);

    #[cfg(feature = "dev")]
    {
        app.add_plugins((
            AnimTestPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
        ));
    }

    app
}

fn configure_rapier(mut query: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    for mut config in &mut query {
        config.gravity = Vec3::ZERO;
        config.physics_pipeline_active = true;
        config.scaled_shape_subdivision = 10;
        config.force_update_from_transform_changes = false;
    }
}
