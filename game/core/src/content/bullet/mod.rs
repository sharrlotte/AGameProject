pub mod physic;
pub mod spawner;

use bevy::prelude::*;
use bevy::{math::Vec2, time::Timer};

use crate::AppState;

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
        app.add_systems(Update, (handle_despawn_bullet, handle_bullet_collisions).run_if(in_state(AppState::Game)))
            .add_systems(OnEnter(AppState::Home), cleanup_bullets);
    }
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
