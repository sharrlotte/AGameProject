use bevy::ecs::component::Component;

#[derive(Component, Clone, PartialEq, Copy, Debug)]
pub enum EntityType {
    Ship,
    Block,
    Projectile,
}
