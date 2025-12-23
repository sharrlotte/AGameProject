use bevy::prelude::*;

use crate::content::{
    block::plugin::BlockPlugin, bullet::plugin::BulletPlugin, ship::plugin::ShipPlugin,
};

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BulletPlugin, ShipPlugin, BlockPlugin));
    }
}
