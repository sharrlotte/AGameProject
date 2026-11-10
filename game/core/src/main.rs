use std::path::Path;

use bevy::audio::AudioPlugin;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_rapier3d::prelude::*;
use combat::CombatPlugin;
use data::CoreDataPlugin;
use enemies::EnemiesPlugin;
use extraction::{ExtractionPlugin, RunFlowPlugin, UpgradesPlugin};
use presentation::AllPresentationPlugins;
use world::{ArenaPlugin, PlayerPlugin};

#[cfg(debug_assertions)]
use bevy::input::common_conditions::input_toggle_active;
#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn resolve_asset_path() -> String {
    if Path::new("assets").exists() {
        "assets".to_string()
    } else {
        "../assets".to_string()
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: resolve_asset_path(),
                watch_for_changes_override: Some(true),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "AGameProject".to_string(),
                    present_mode: PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AudioPlugin::default()),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .insert_resource(TimestepMode::Interpolated {
        dt: 1.0 / 60.0,
        time_scale: 1.0,
        substeps: 1,
    })
    .add_plugins((
        FrameTimeDiagnosticsPlugin::new(100),
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

    #[cfg(debug_assertions)]
    {
        app.add_plugins(EguiPlugin::default()).add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
        );
    }

    app.run();
}

fn configure_rapier(mut query: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    for mut config in &mut query {
        config.gravity = Vec3::ZERO;
        config.physics_pipeline_active = true;
        config.scaled_shape_subdivision = 10;
        config.force_update_from_transform_changes = false;
    }
}
