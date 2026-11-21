pub mod ai;
pub mod component;
pub mod inspect;
pub mod spawn;
pub mod stat;
pub mod system;
pub mod throttle;
pub mod vision;
pub use component::*;
pub use inspect::*;
pub use stat::*;
pub use system::*;
pub use throttle::*;
pub use vision::*;

mod indicator;

use bevy::prelude::*;

use crate::content::block::stat::StatsUpdateSet;
use crate::content::ship::stat::sync_ship_health;
use crate::content::ship::ai::AiPlugin;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

use indicator::draw_indicators;

#[cfg(feature = "dev")]
pub mod debug;

#[cfg(any(feature = "dev", test))]
pub mod geometry;

#[cfg(feature = "dev")]
pub use geometry::*;

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
            FixedUpdate,
            (update_ship_steering, collect_drop)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Control),
        )
        .add_systems(
            FixedPreUpdate,
            (
                sync_ship_health,
                update_stats,
            )
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
        .add_plugins((IndicatorsPlugin, AiPlugin, ShipInspectPlugin, VisionPlugin));
    }
}
