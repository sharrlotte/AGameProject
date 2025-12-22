pub mod base;
pub mod registry;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    components::death::Death,
    content::block::{base::{Block, BlockType, Health, ShipIdComponent}, registry::register_default_blocks},
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BlockDestroyedEvent>()
            .add_systems(Last, handle_block_destroy)
            .add_systems(PreStartup, register_default_blocks);
    }
}

#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block: BlockType,
    pub block_id: Uuid,
    pub ship_id: Entity,
}

fn handle_block_destroy(
    mut commands: Commands,
    mut writer: MessageWriter<BlockDestroyedEvent>,
    mut query: Query<(Entity, &Health, &Block, &ShipIdComponent)>,
) {
    for (entity, health, block, ship_id) in query.iter_mut() {
        if health.health <= 0.0 {
            writer.write(BlockDestroyedEvent {
                block: block.block_type,
                ship_id: ship_id.id,
                block_id: block.id,
            });
            commands.entity(entity).insert(Death);
        }
    }
}
