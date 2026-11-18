pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::SceneState;
use crate::simulation::SimulationSet;

use system::{spawn_damage_text_pool, tick_damage_text_pool};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(OnEnter(SceneState::Game), spawn_damage_text_pool)
            .add_systems(
                FixedUpdate,
                handle_damage_collisions
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Damage),
            )
            .add_systems(
                FixedPostUpdate,
                handle_damage_events
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Damage),
            )
            .add_systems(Update, tick_damage_text_pool.in_set(SimulationSet::Present));
    }
}
