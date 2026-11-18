pub mod ai;
pub mod component;
pub mod grid;
pub mod mutation;
pub mod stat;
pub mod system;
pub mod throttle;
pub use component::*;
pub use grid::*;
pub use mutation::*;
pub use stat::*;
pub use system::*;
pub use throttle::*;

mod indicator;

use bevy::prelude::*;

use crate::content::block::handle_block_destroy;
use crate::content::block::stat::StatsUpdateSet;
use crate::content::ship::ai::AiPlugin;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

use indicator::draw_indicators;

pub struct IndicatorsPlugin;

impl Plugin for IndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_indicators.in_set(SimulationSet::Present));
    }
}

pub struct ShipMutationPlugin;

impl Plugin for ShipMutationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShipMutation>()
            .add_message::<ShipChanged>()
            .add_systems(
                FixedPostUpdate,
                apply_ship_mutations
                    .after(handle_block_destroy)
                    .in_set(SimulationSet::Death),
            );
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShipStats>()
            .register_type::<Throttle>();
        #[cfg(debug_assertions)]
        app.add_systems(
            FixedUpdate,
            update_ship_ui
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        );
        app.add_systems(
            PostUpdate,
            update_ship_behavior
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        )
        .add_systems(
            FixedPreUpdate,
            update_stats
                .after(StatsUpdateSet)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Stats),
        )
        .add_systems(
            FixedPreUpdate,
            rebuild_cargo_capacity
                .run_if(on_message::<ShipChanged>)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Stats),
        )
        .add_systems(
            FixedUpdate,
            collect_drop
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Control),
        )
        .add_plugins((IndicatorsPlugin, AiPlugin, ShipMutationPlugin));
    }
}
