use bevy::prelude::*;
use data::{
    ConfigHotReloadedMessage, PurchaseUpgradeMessage, ScrapBank, UpgradeLevels, UpgradeModifiers,
    UpgradesChangedMessage,
};

use super::config::{UpgradeLevelsExt, UpgradeModifiersExt, UpgradesConfig};

pub fn handle_upgrade_purchases(
    mut purchase_reader: MessageReader<PurchaseUpgradeMessage>,
    cfg: Res<UpgradesConfig>,
    mut bank: ResMut<ScrapBank>,
    mut levels: ResMut<UpgradeLevels>,
    mut modifiers: ResMut<UpgradeModifiers>,
    mut changed_writer: MessageWriter<UpgradesChangedMessage>,
) {
    let mut bought = false;
    for msg in purchase_reader.read() {
        let lvl = levels.level(msg.kind);
        let max_lvl = levels.max_level(msg.kind, &cfg);
        let cost = levels.next_cost(msg.kind, &cfg);
        if lvl < max_lvl && bank.banked >= cost {
            bank.banked -= cost;
            levels.increment(msg.kind);
            bought = true;
        }
    }

    if bought {
        modifiers.recompute(&levels, &cfg);
        changed_writer.write(UpgradesChangedMessage);
    }
}

pub fn recompute_upgrades_on_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<UpgradesConfig>,
    levels: Res<UpgradeLevels>,
    mut modifiers: ResMut<UpgradeModifiers>,
) {
    if reloaded.read().next().is_some() && cfg.is_changed() {
        modifiers.recompute(&levels, &cfg);
    }
}
