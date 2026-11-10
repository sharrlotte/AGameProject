use bevy::prelude::*;

pub mod camera;
pub mod hud;
pub mod input;
pub mod menus;
pub mod player_visuals;
pub mod visuals;

pub use camera::CameraPlugin;
pub use hud::HudPlugin;
pub use input::InputPlugin;
pub use menus::MenusPlugin;
pub use player_visuals::PlayerVisualsPlugin;
pub use visuals::VisualsPlugin;

/// Convenience group registering all presentation feature plugins.
pub struct AllPresentationPlugins;

impl Plugin for AllPresentationPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraPlugin,
            InputPlugin,
            VisualsPlugin,
            PlayerVisualsPlugin,
            HudPlugin,
            MenusPlugin,
        ));
    }
}
