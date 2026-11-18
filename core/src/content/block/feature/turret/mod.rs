pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::content::block::feature::power::PowerSet;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedFirst, update_turret_demand.in_set(PowerSet::Demand))
            .add_systems(
                FixedUpdate,
                (
                    advance_turret_charge,
                    turret_tracking,
                    update_turret_target,
                )
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Weapons),
            );
    }
}
