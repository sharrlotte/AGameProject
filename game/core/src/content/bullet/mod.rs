use bevy::prelude::*;
use bevy::{math::Vec2, time::Timer};

#[derive(Component, Debug, Clone, Reflect)]
pub struct Bullet {
    pub timer: Timer,
    pub damage: f32,
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: u8,
}
