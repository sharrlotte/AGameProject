use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use data::{AppRonConfigExt, GameState, GameScene, SimPostPhysicsSet, SimUpdateSet};

use crate::config::PlayerConfig;

pub mod state;
pub mod system;

pub use state::{AimDirection, Health, LocalPlayer, MovementStats, Stamina, Survivor};
pub use system::{apply_player_damage, player_movement_and_aim, spawn_player};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<PlayerConfig>()
            .add_systems(OnEnter(GameState::Playing), system::spawn_player)
            .add_systems(
                OnEnter(GameScene::Arena),
                system::spawn_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), system::freeze_survivor_on_game_over)
            .add_systems(
                Update,
                (system::sync_player_hot_reload, system::player_movement_and_aim)
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                system::apply_player_damage
                    .in_set(SimPostPhysicsSet)
                    .in_set(data::DamageResolutionSet)
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
