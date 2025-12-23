use bevy::prelude::*;

use crate::save::{
    io::{handle_load_world_event, handle_save_world_event},
    message::{LoadWorldEvent, SaveWorldEvent},
};
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .add_systems(
                Update,
                (
                    handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
                    handle_load_world_event.run_if(on_message::<LoadWorldEvent>),
                ),
            );
    }
}
