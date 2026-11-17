pub mod component;

pub use component::*;

use bevy::prelude::*;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlMode>()
            .init_resource::<CameraTarget>()
            .init_resource::<ControlledShip>();
    }
}
