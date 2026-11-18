pub mod block;
pub mod bullet;
pub mod damage;
pub mod death;
pub mod item;
pub mod lifetime;
pub mod selectable;
pub mod ship;
pub mod team;

use bevy::prelude::*;

use crate::content::{
    block::BlockPlugin, bullet::BulletPlugin, damage::DamagePlugin, death::DeathPlugin,
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
