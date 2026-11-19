mod component;
mod lib;
mod system;

pub use component::*;
pub use lib::*;

use bevy::prelude::*;

use crate::simulation::SimulationSet;
use system::{on_scroll_handler, send_scroll_events, sync_scrollbars};

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                send_scroll_events.in_set(SimulationSet::Input),
                sync_scrollbars.in_set(SimulationSet::Present),
            ),
        )
        .add_observer(on_scroll_handler);
    }
}
