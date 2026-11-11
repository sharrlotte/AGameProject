use bevy::prelude::*;

/// Physical 3D projectile sensor entity fired by weapons.
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub owner: Entity,
    pub damage: f32,
    pub knockback: f32,
    pub direction: Vec3,
    pub radius: f32,
    pub lifetime: f32,
    pub hit: bool,
}
