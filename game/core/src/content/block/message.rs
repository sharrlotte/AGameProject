use bevy::prelude::*;
use uuid::Uuid;

use crate::content::block::resource::BlockType;


#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block_type: BlockType,
    pub block_id: Uuid,
    pub ship: Entity,
}
