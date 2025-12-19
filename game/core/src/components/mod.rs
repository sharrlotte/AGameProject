use bevy::ecs::component::Component;
use uuid::Uuid;

pub mod death;

#[derive(Component)]
pub struct Id(pub Uuid);
