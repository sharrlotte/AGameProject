pub mod physic;
pub mod spawner;
pub mod types;

use bevy::prelude::*;

use crate::AppState;
use crate::content::bullet::physic::{handle_bullet_collisions, handle_despawn_bullet};

pub use types::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_despawn_bullet, handle_bullet_collisions).run_if(in_state(AppState::Game)),
        )
        .add_systems(OnEnter(AppState::Home), cleanup_bullets);
    }
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
