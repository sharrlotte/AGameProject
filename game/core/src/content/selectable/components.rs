use std::collections::HashMap;
use bevy::prelude::*;
use crate::content::{entity::EntityType, ship::BoundingBox};

#[derive(Component)]
pub struct Selectable;

#[derive(Resource, Default)]
pub struct SelectableRes {
    pub state: SelectableState,
}

#[derive(Message)]
pub struct SelectedEvent {
    pub selected: HashMap<EntityType, Vec<Entity>>,
}

#[derive(Default)]
pub enum SelectableState {
    #[default]
    Unselected,
    Selecting {
        start_position: Vec2,
    },
}
