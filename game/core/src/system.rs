use bevy::post_process::bloom::BloomCompositeMode;
use bevy::prelude::*;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

use crate::resource::{GameVars, SceneEntity, SceneState};
use crate::save::io::load_saved_schematics;

pub fn setup_camera(mut commands: Commands) {
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
        SpatialListener::new(400.0),
    ));
}

pub fn load_current_ship(mut state: ResMut<GameVars>) {
    let ships = load_saved_schematics();

    if let Some(ship) = ships.first() {
        state.current_ship = Some(ship.clone());
    }
}

pub fn clean_state_item(
    mut commands: Commands,
    mut events: MessageReader<StateTransitionEvent<SceneState>>,
    mut scene_entities: Query<(Entity, &SceneEntity)>,
) {
    for event in events.read() {
        let Some(exited) = event.exited else {
            continue;
        };

        for (entity, scene_entity) in scene_entities.iter_mut() {
            if scene_entity.0 == exited {
                commands.entity(entity).despawn();
            }
        }
    }
}
