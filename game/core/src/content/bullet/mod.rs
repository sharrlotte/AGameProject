pub mod base;
pub mod physic;

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::SceneState;
use crate::components::entity_type::EntityType;
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
            handle_despawn_bullet.run_if(in_state(SceneState::Game)),
        )
        .add_systems(Update, log_projectiles.run_if(on_timer(Duration::from_millis(500))))
        .add_systems(OnEnter(SceneState::Home), cleanup_bullets);
    }
}

fn log_projectiles(bullets: Query<&EntityType>) {
    info!("entities: {}", bullets.iter().count());
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<EntityType>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
