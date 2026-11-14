use bevy::prelude::*;
use crate::common::GameState;
use crate::config::AppRonConfigExt;

pub mod upgrade;

pub use upgrade::{handle_upgrade_purchases, recompute_upgrades_on_hot_reload, UpgradeLevelsExt, UpgradeModifiersExt, UpgradesConfig};

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<UpgradesConfig>().add_systems(
            Update,
            (
                upgrade::handle_upgrade_purchases.run_if(in_state(GameState::Upgrades)),
                upgrade::recompute_upgrades_on_hot_reload,
            ),
        );
    }
}
