use bevy::prelude::*;
use data::{GameState, GameScene};

use crate::arena::{ArenaGround, system::spawn_ground};
use crate::player::spawn_player;

/// Dev-only enemy-free flat test stage plugin for inspecting animations cleanly.
pub struct AnimTestPlugin;

impl Plugin for AnimTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameScene::AnimTest),
            (
                spawn_anim_test_ground,
                spawn_player.run_if(in_state(GameState::Playing)),
            )
                .run_if(not(in_state(GameState::Loading))),
        );
    }
}

pub fn spawn_anim_test_ground(mut commands: Commands, existing: Query<(), With<ArenaGround>>) {
    if !existing.is_empty() {
        return;
    }
    spawn_ground(&mut commands, GameScene::AnimTest);
}
