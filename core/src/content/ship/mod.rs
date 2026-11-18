pub mod ai;
#[cfg(feature = "dev")]
pub mod debug;
pub mod component;
pub mod geometry;
pub mod stat;
pub mod system;
pub mod throttle;
pub use component::*;
pub use geometry::*;
pub use stat::*;
pub use system::*;
pub use throttle::*;

mod indicator;

use bevy::prelude::*;

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

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShipStats>()
            .register_type::<Throttle>()
            .add_message::<ShipChanged>();
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
        .add_plugins((IndicatorsPlugin, AiPlugin));
    }
}
