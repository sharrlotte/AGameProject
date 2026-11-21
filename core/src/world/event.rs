use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SectorDiscovered {
    pub position: Vec2,
    pub chunk: IVec2,
}
