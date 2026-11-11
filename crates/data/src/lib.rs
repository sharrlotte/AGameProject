pub use common::*;
pub use config::*;
pub use message::*;

use bevy::prelude::*;

/// Compatibility umbrella plugin that registers config, common, and message systems/resources.
pub struct CoreDataPlugin;

impl Plugin for CoreDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(MessagePlugin)
            .add_plugins(ConfigPlugin);
    }
}
