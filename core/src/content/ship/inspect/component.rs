use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct InspectedShip(pub Option<Entity>);

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipSelected {
    pub entity: Entity,
    pub toggle: bool,
}

impl ShipSelected {
    pub fn toggle(entity: Entity) -> Self {
        Self {
            entity,
            toggle: true,
        }
    }

    pub fn select(entity: Entity) -> Self {
        Self {
            entity,
            toggle: false,
        }
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipSelectionCleared;
