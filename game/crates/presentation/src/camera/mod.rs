use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use data::GameState;

pub mod system;

pub use system::MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), system::spawn_camera)
            .add_systems(
                Update,
                system::sync_camera_on_hot_reload.run_if(not(in_state(GameState::Loading))),
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
