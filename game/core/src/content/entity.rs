use bevy::ecs::component::Component;

#[derive(Component, Clone, PartialEq, Copy, Debug, Eq, Hash)]
pub enum EntityType {
    Ship,
    Block,
    Bullet,
    Missile,
    Laser,
    Ammo
}
