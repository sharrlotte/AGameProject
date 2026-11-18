mod component;
mod system;

pub use component::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;

use system::{setup_assets, update_ammo, update_no_ammo_warning};

pub struct AmmoPlugin;

impl Plugin for AmmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_ammo.in_set(SimulationSet::Weapons),
                update_no_ammo_warning.in_set(SimulationSet::Present),
            ),
        )
        .add_systems(Startup, setup_assets);
    }
}
