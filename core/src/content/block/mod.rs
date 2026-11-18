pub mod component;
pub mod feature;
pub mod placement;
pub mod radar;
pub mod stat;
pub mod system;

pub use component::*;
pub use placement::*;
pub use system::*;

use bevy::prelude::*;

use crate::content::block::feature::ComponentPlugin;
use crate::content::block::radar::RadarPlugin;
use crate::content::ship::apply_ship_mutations;
use crate::simulation::SimulationSet;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_message::<DropEvent>()
            .add_systems(
                FixedPostUpdate,
                (
                    handle_block_damage_event,
                    handle_flash_overlay,
                    handle_block_destroy,
                )
                    .in_set(SimulationSet::Death),
            )
            .add_systems(
                FixedPostUpdate,
                handle_drop
                    .after(apply_ship_mutations)
                    .in_set(SimulationSet::Death),
            )
            .add_plugins((RadarPlugin, ComponentPlugin));
    }
}
