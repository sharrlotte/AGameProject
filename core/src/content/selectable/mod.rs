pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;

use system::update_selectable;

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectableRes>()
            .add_message::<EntitySelectEvent>()
            .add_systems(Update, update_selectable.in_set(SimulationSet::Input));
    }
}
