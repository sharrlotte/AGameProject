use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        schedule::{IntoScheduleConfigs, common_conditions::on_message},
        world::World,
    },
};

use crate::save::{base::SaveWorldEvent, io::handle_save_world_event};

pub mod base;
pub mod io;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>().add_systems(
            Update,
            handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
        );
    }
}
