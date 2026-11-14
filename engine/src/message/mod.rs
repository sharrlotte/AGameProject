use bevy::prelude::*;

pub mod messages;
pub mod resources;

pub use messages::*;
pub use resources::*;

pub struct MessagePlugin;

impl Plugin for MessagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::RunStats>()
            .init_resource::<resources::ScrapBank>()
            .init_resource::<resources::LastRunOutcome>()
            .init_resource::<resources::UpgradeLevels>()
            .init_resource::<resources::UpgradeModifiers>()
            .add_message::<messages::DamageMessage>()
            .add_message::<messages::EnemyKilledMessage>()
            .add_message::<messages::DetonationMessage>()
            .add_message::<messages::WeaponFiredMessage>()
            .add_message::<messages::PlayerDiedMessage>()
            .add_message::<messages::ReloadStartedMessage>()
            .add_message::<messages::ReloadFinishedMessage>()
            .add_message::<messages::ReloadCancelledMessage>()
            .add_message::<messages::WeaponSwappedMessage>()
            .add_message::<messages::ScrapCollectedMessage>()
            .add_message::<messages::ExtractionCompletedMessage>()
            .add_message::<messages::PurchaseUpgradeMessage>()
            .add_message::<messages::UpgradesChangedMessage>()
            .add_message::<messages::DeployRequestedMessage>()
            .add_message::<messages::OpenUpgradesRequestedMessage>();
    }
}
