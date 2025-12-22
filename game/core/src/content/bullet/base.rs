use bevy::prelude::*;

use crate::save::base::Save;

#[derive(Component, Debug, Clone, Reflect)]
#[require(Save)]
pub struct Bullet {
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: i8,
    pub velocity: Vec2,
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(Save)]
pub struct Missile {
    pub lock_range: f32,
    pub shooted_at: Vec2,
    pub speed: f32,
    pub target: Option<Entity>,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct MissileHitbox {
    pub missile: Entity,
}
