pub mod physic;
pub mod spawner;

use bevy::prelude::*;
use bevy::{math::Vec2, time::Timer};

use crate::content::bullet::physic::{handle_bullet_collisions, handle_despawn_bullet};

#[derive(Component, Debug, Clone, Reflect)]
pub struct Bullet {
    pub timer: Timer,
    pub damage: f32,
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: u8,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_despawn_bullet, handle_bullet_collisions));
    }
}
