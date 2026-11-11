pub use common::*;
pub use config::*;
pub use message::*;

/// Compatibility umbrella plugin that registers config, common, and message systems/resources.
pub struct CoreDataPlugin;

impl bevy::prelude::Plugin for CoreDataPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use bevy::prelude::AppExtStates;

        app.init_state::<common::GameScene>()
            .init_state::<common::GameState>()
            .add_plugins(MessagePlugin)
            .add_plugins(ConfigPlugin);
    }
}
