use bevy::ecs::{entity::Entity, resource::Resource};

#[derive(Default, Resource)]
pub struct GameVars {
    pub controlling: Option<Entity>,
    pub following: Option<Entity>,
}
