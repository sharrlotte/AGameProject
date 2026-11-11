use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use data::{AppRonConfigExt, GameState, GameScene, SimPostPhysicsSet, SimUpdateSet};

pub mod ai;
pub mod config;
pub mod damage;
pub mod spawn;

pub use ai::{Enemy, FacingDirection, enemy_chase_and_attack};
pub use config::{EnemiesConfig, EnemyArchetypeData};
pub use damage::apply_enemy_damage;
pub use spawn::{EnemySpawner, escalated_spawn_params, spawn_enemies, spawn_enemy_entity};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<EnemiesConfig>()
            .init_resource::<EnemySpawner>()
            .add_systems(
                OnEnter(GameState::Playing),
                spawn::reset_spawner.run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                OnEnter(GameScene::Arena),
                spawn::reset_spawner.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnEnter(GameState::GameOver),
                ai::freeze_enemies_on_game_over.run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                Update,
                (
                    ai::sync_enemies_hot_reload,
                    spawn::spawn_enemies,
                    ai::enemy_chase_and_attack,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                PostUpdate,
                damage::apply_enemy_damage
                    .in_set(SimPostPhysicsSet)
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            );
    }
}

#[cfg(test)]
mod tests;
