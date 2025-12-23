use bevy::prelude::*;

use crate::content::block::{
    message::BlockDestroyedEvent,
    radar::plugin::RadarPlugin,
    resource::{BlockRegistry, register_default_blocks},
    system::{handle_block_damage_event, handle_block_destroy, handle_flash_overlay},
    turret::plugin::TurretPlugin,
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_message::<BlockDestroyedEvent>()
            .add_systems(PreStartup, register_default_blocks)
            .add_systems(Last, (handle_block_damage_event, handle_flash_overlay))
            .add_systems(Last, handle_block_destroy)
            .add_plugins((RadarPlugin, TurretPlugin));
    }
}
