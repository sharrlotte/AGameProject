use bevy::prelude::*;

pub mod anim;
pub mod camera;
pub mod config;
pub mod hud;
pub mod input;
pub mod menus;
pub mod visuals;

pub use anim::PlayerAnimPlugin;
pub use camera::CameraPlugin;
pub use config::{PlayerModelConfig, VisualsConfig};
pub use hud::HudPlugin;
pub use input::InputPlugin;
pub use menus::MenusPlugin;
pub use visuals::VisualsPlugin;

/// Convenience group registering all presentation feature plugins.
pub struct AllPresentationPlugins;

impl Plugin for AllPresentationPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraPlugin,
            InputPlugin,
            VisualsPlugin,
            PlayerAnimPlugin,
            HudPlugin,
            MenusPlugin,
        ));
    }
}
