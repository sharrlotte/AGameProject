use bevy::prelude::*;

use crate::common::lifetime::system::update_lifetime;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_lifetime);
    }
}
