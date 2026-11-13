use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use crate::config::AppRonConfigExt;
use crate::common::GameState;
use crate::common::SimUpdateSet;

pub mod state;
pub mod system;

pub use crate::config::CameraConfig;
pub use state::{CameraZoom, CameraZoomMessage, MainCamera};

/// Manages the main 3D gameplay camera, hot-reload sync, follow damping, and zoom.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<CameraConfig>()
            .add_message::<CameraZoomMessage>()
            .add_systems(OnExit(GameState::Loading), system::spawn_camera)
            .add_systems(
                Update,
                (
                    system::sync_camera_on_hot_reload
                        .after(SimUpdateSet)
                        .run_if(not(in_state(GameState::Loading))),
                    system::apply_camera_zoom
                        .after(SimUpdateSet)
                        .run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(
                PostUpdate,
                system::follow_local_player
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}







