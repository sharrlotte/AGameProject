use bevy::prelude::*;

pub mod events;
pub mod loader;
pub mod state;

pub use events::*;
pub use loader::{AppRonConfigExt, ConfigSyncSet, DataPlugin, RonConfig};
pub use state::*;

/// Registers shared state, resources, messages, and `.ron` asset loaders.
pub struct CoreDataPlugin;

impl Plugin for CoreDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<RunStats>()
            .init_resource::<ScrapBank>()
            .init_resource::<LastRunOutcome>()
            .init_resource::<UpgradeLevels>()
            .init_resource::<UpgradeModifiers>()
            .add_message::<DamageMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_message::<WeaponFiredMessage>()
            .add_message::<PlayerDiedMessage>()
            .add_message::<ScrapCollectedMessage>()
            .add_message::<ExtractionCompletedMessage>()
            .add_message::<PurchaseUpgradeMessage>()
            .add_message::<UpgradesChangedMessage>()
            .add_message::<DeployRequestedMessage>()
            .add_message::<OpenUpgradesRequestedMessage>()
            .add_message::<ConfigHotReloadedMessage>()
            .add_plugins(DataPlugin);
    }
}
