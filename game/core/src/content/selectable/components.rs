use crate::content::entity::EntityType;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Selectable;

#[derive(Resource, Default)]
pub struct SelectableRes {
    pub state: SelectableState,
}

#[derive(Message)]
pub struct EntitySelectEvent {
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
