use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use crate::config::AppRonConfigExt;
use crate::common::{
    DamageResolutionSet, DeathResolutionSet, GameScene, GameState, SimPostPhysicsSet, SimUpdateSet,
};

pub mod ai;
pub mod brain;
pub mod damage;
pub mod death;
pub mod detonate;
pub mod melee;
pub mod ranged;
pub mod spawn;

pub use ai::{Enemy, FacingDirection, steer_enemies, sync_enemies_hot_reload};
pub use brain::{BrainState, EnemyBrain};
pub use detonate::{Detonated, Fuse};
pub use melee::Melee;
pub use ranged::{AcidProjectile, Ranged};
pub use crate::config::{
    ArmorSpec, Behavior, DetonateSpec, EnemiesConfig, EnemyArchetypeData, MeleeSpec, RangedSpec,
    SpawnPolicy,
};
pub use damage::apply_enemy_damage;
pub use death::cleanup_dead;
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
                    brain::update_brains,
                    melee::enemy_melee,
                    ranged::enemy_ranged,
                    detonate::update_fuses,
                    ai::steer_enemies,
                    ranged::fall_acid_projectiles,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            )
            .add_systems(
                PostUpdate,
                (
                    ranged::resolve_acid_projectiles,
                    damage::apply_enemy_damage.in_set(DamageResolutionSet),
                    detonate::resolve_detonations
                        .in_set(DeathResolutionSet)
                        .before(death::cleanup_dead),
                    death::cleanup_dead.in_set(DeathResolutionSet),
                )
                    .chain()
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
