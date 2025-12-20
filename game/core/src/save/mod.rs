use bevy::{
    app::{App, Plugin, PostUpdate, Update},
    ecs::schedule::{IntoScheduleConfigs, common_conditions::on_message},
};

use crate::save::{
    base::{LoadWorldEvent, SaveWorldEvent},
    io::{handle_load_world_event, handle_save_world_event},
};

pub mod base;
pub mod io;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .add_systems(
                Update,
                handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
            )
            .add_systems(
                PostUpdate,
                handle_load_world_event.run_if(on_message::<LoadWorldEvent>),
            );
    }
}
