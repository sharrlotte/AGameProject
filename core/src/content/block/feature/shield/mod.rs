pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;

use crate::simulation::SimulationSet;

pub struct ShieldPlugin;

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ShieldMaterial>::default())
            .register_type::<Shield>()
            .register_type::<Barrier>()
            .add_systems(
                FixedUpdate,
                (spawn_shield_barriers, update_shield_state)
                    .chain()
                    .in_set(SimulationSet::Weapons),
            )
            .add_systems(Update, update_shield_visuals.in_set(SimulationSet::Present));
    }
}
