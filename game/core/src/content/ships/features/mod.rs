pub mod shooting;
pub mod tracking;

use bevy::{
    app::{App, Plugin, Update},
    camera::{Camera, Camera2d},
    ecs::{
        query::With,
        resource::Resource,
        system::{ResMut, Single},
    },
    math::Vec2,
    transform::components::GlobalTransform,
    window::{PrimaryWindow, Window},
};

use crate::content::ships::features::{
    shooting::ShootingFeaturePlugin, tracking::TrackingFeaturePlugin,
};

pub struct ShipFeature;

impl Plugin for ShipFeature {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeatureState>()
            .add_systems(Update, process_input)
            .add_plugins(TrackingFeaturePlugin)
            .add_plugins(ShootingFeaturePlugin);
    }
}

#[derive(Resource, Default)]
pub struct FeatureState {
    cursor_world_pos: Option<Vec2>,
}

fn process_input(
    mut state: ResMut<FeatureState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) {
            state.cursor_world_pos = Some(world_pos);
        }
    }
}
