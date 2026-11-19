use bevy::prelude::*;

#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    pub(super) entity: Entity,
    pub(super) delta: Vec2,
}

#[derive(Component)]
pub struct ScrollViewport;

#[derive(Component)]
pub struct ScrollTarget(pub Entity);

#[derive(Component)]
pub struct ScrollbarTrack;

#[derive(Component)]
pub struct ScrollbarThumb;
