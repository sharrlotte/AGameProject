pub mod block;
pub mod body;
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
    block::BlockPlugin, body::BodyPlugin, bullet::BulletPlugin, damage::DamagePlugin,
    death::DeathPlugin, item::ItemPlugin, lifetime::LifetimePlugin, selectable::SelectablePlugin,
    ship::ShipPlugin,
};

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BulletPlugin,
            ShipPlugin,
            BodyPlugin,
            BlockPlugin,
            ItemPlugin,
            LifetimePlugin,
            DeathPlugin,
            DamagePlugin,
            SelectablePlugin,
        ));
    }
}
