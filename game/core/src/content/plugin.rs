use bevy::prelude::*;

use crate::content::{
    block::systems::BlockPlugin, bullet::BulletPlugin, damage::DamagePlugin, death::DeathPlugin,
    item::ItemPlugin, lifetime::LifetimePlugin, selectable::SelectablePlugin, ship::ShipPlugin,
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
