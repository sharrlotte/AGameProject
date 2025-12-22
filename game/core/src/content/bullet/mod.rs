pub mod base;
pub mod physic;

use bevy::prelude::*;

use crate::SceneState;
use crate::components::entity_type::EntityType;
use crate::content::bullet::physic::{
    handle_bullet_collisions, handle_bullets, handle_burst_shot, handle_despawn_bullet,
    handle_missile_radar, handle_missiles, handle_single_shot,
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
                handle_single_shot,
                handle_burst_shot,
            )
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            PostUpdate,
            handle_despawn_bullet.run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup_bullets);
    }
}

fn cleanup_bullets(mut commands: Commands, bullets: Query<Entity, With<EntityType>>) {
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
}
