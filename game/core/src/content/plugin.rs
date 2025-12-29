use bevy::prelude::*;

use crate::content::{
    block::base::BlockPlugin, bullet::base::BulletPlugin, item::base::ItemPlugin,
    ship::base::ShipPlugin,
};

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BulletPlugin, ShipPlugin, BlockPlugin, ItemPlugin));
    }
}
