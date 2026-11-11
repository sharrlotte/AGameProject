use bevy::prelude::*;

pub mod hud;
pub mod menu;
pub mod theme;

pub use hud::HudPlugin;
pub use menu::MenusPlugin;

/// Registers all HUD and menu UI plugins.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((HudPlugin, MenusPlugin));
    }
}
