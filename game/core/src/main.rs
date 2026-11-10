use bevy::audio::AudioPlugin;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowPlugin};
use bevy_rapier2d::prelude::*;

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
                ..default()
            }),
    )
    .add_plugins(
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0)
            .in_fixed_schedule(),
    )
    // .add_plugins(RapierDebugRenderPlugin::default())
    .add_plugins((
        FrameTimeDiagnosticsPlugin::new(100),
        EntityCountDiagnosticsPlugin::new(100),
    ))
    .insert_resource(Time::<Fixed>::from_hz(60.0))
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
        config.gravity = Vec2::ZERO;
        config.physics_pipeline_active = true;
        config.scaled_shape_subdivision = 10;
        config.force_update_from_transform_changes = false;
    }
}
