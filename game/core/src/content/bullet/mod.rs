pub mod base;
pub mod physic;

use bevy::prelude::*;

use crate::SceneState;
use crate::content::bullet::base::Bullet;
use crate::content::bullet::physic::{handle_bullet_collisions, handle_despawn_bullet};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_despawn_bullet, handle_bullet_collisions).run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup_bullets);
    }
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
