pub mod board;
pub mod capability;
pub mod component;
pub mod config;
pub mod readiness;
pub mod scorer;
pub mod sensor;
pub mod steer;

pub use board::*;
pub use capability::*;
pub use component::*;
pub use config::*;
pub use readiness::*;
pub use scorer::*;
pub use sensor::*;
pub use steer::*;

use bevy::prelude::*;

use crate::content::damage::DamageEvent;
use crate::content::ship::ShipChanged;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiConfigValue>()
            .init_resource::<TacticalBoard>()
            .add_systems(
                FixedUpdate,
                (
                    init_missing_capabilities,
                    rebuild_capabilities_on_change.run_if(on_message::<ShipChanged>),
                    refresh_quadrant_hp_on_damage.run_if(on_message::<DamageEvent>),
                    update_readiness,
                    update_sensors_and_intel,
                    update_tactical_board,
                    evaluate_utility_and_power,
                    steer_ships,
                )
                    .chain()
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Ai),
            );
    }
}

#[cfg(test)]
mod tests;
