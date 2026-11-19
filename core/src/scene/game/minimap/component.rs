use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapMarkerLayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapMarker {
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkerSpec {
    pub offset: Vec2,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LostContact {
    pub entity: Entity,
    pub world_position: Vec2,
    pub color: Color,
    pub elapsed: f32,
}

#[derive(Resource, Default, Debug)]
pub struct MinimapState {
    pub prev_contacts: HashMap<Entity, (Vec2, Color)>,
    pub lost_flashes: Vec<LostContact>,
}
