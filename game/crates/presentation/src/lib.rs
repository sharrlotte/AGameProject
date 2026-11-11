use bevy::prelude::*;

pub mod anim;
pub mod camera;
pub mod config;
pub mod input;
pub mod scene;
pub mod ui;
pub mod vfx;

pub use anim::PlayerAnimPlugin;
pub use camera::CameraPlugin;
pub use config::{PlayerModelConfig, VisualsConfig};
pub use input::InputPlugin;
pub use scene::ScenePlugin;
pub use ui::{HudPlugin, MenusPlugin, UiPlugin};
pub use vfx::VfxPlugin;

/// Convenience group registering all presentation feature plugins.
pub struct AllPresentationPlugins;

impl Plugin for AllPresentationPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraPlugin,
            InputPlugin,
            ScenePlugin,
            VfxPlugin,
            PlayerAnimPlugin,
            UiPlugin,
        ));
    }
}
