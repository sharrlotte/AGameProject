use bevy::prelude::*;

use crate::{
    SceneState,
    content::bullet::system::{
        cleanup_bullets, destroy_out_of_range_bullet, handle_bullets, handle_burst_shot,
        handle_missile_radar_target, handle_missiles, handle_single_shot, update_bullet_lifetime,
    },
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
            (destroy_out_of_range_bullet, update_bullet_lifetime)
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup_bullets);
    }
}
