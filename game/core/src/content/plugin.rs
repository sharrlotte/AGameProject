use bevy::prelude::*;

use crate::content::{
    block::base::BlockPlugin, bullet::base::BulletPlugin, damage::DamagePlugin, death::DeathPlugin,
    item::base::ItemPlugin, lifetime::LifetimePlugin, selectable::SelectablePlugin,
    ship::base::ShipPlugin,
};

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BulletPlugin,
            ShipPlugin,
            BlockPlugin,
            ItemPlugin,
            LifetimePlugin,
            DeathPlugin,
            DamagePlugin,
            SelectablePlugin,
        ));
    }
}
