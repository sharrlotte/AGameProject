use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Projectile;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Bullet {
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: i8,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Missile {
    pub range: f32,
    pub shooted_at: Vec2,
    pub speed: f32,
    pub target: Option<Entity>,
}
