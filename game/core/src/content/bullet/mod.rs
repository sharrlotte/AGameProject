pub mod base;
pub mod system;

use bevy::prelude::*;

use crate::SceneState;
use crate::components::death::Death;
use crate::components::entity_type::EntityType;
use crate::content::bullet::base::LifetimePlugin;
use crate::content::bullet::system::{
    destroy_out_of_range_bullet, handle_bullets, handle_burst_shot, handle_missile_radar_target, handle_missiles, handle_single_shot
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                handle_missiles,
                handle_bullets,
                handle_single_shot,
                handle_burst_shot,
                handle_missile_radar_target,
            )
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            destroy_out_of_range_bullet.run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup)
        .add_plugins(LifetimePlugin);
    }
}

fn cleanup(mut commands: Commands, bullets: Query<Entity, With<EntityType>>) {
    for entity in &bullets {
        commands.entity(entity).insert(Death);
    }
}
