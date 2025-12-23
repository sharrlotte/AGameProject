use bevy::prelude::*;
use uuid::Uuid;

use crate::content::block::component::BlockType;

#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block: BlockType,
    pub block_id: Uuid,
    pub ship_id: Entity,
}
