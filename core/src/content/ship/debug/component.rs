use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct ShipDebugSelection {
    pub selected: EntityHashSet,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipDebugSelected {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipDebugSelectionCleared;

#[derive(Component, Debug)]
pub struct ShipUIRoot {
    pub ship: Entity,
}
