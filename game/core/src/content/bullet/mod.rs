pub mod base;
pub mod physic;

use bevy::prelude::*;

use crate::SceneState;
use crate::content::bullet::base::Bullet;
use crate::content::bullet::physic::{
    handle_bullet_collisions, handle_bullets, handle_despawn_bullet, handle_missile_radar,
    handle_missiles,
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_bullet_collisions,
                handle_missiles,
                handle_missile_radar,
                handle_bullets,
            )
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            PostUpdate,
            (handle_despawn_bullet,).run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup_bullets);
    }
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
