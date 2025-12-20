use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        schedule::{IntoScheduleConfigs, common_conditions::on_message},
        world::World,
    },
};

use crate::save::{base::SaveWorldEvent, io::save_world};

pub mod base;
pub mod io;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>().add_systems(
            Update,
            save_world_system.run_if(on_message::<SaveWorldEvent>),
        );
    }
}

fn save_world_system(world: &mut World) {
    match save_world(world, &"Test.ron".to_string()) {
        Ok(_) => {
            println!("Save world success");
        }
        Err(e) => {
            println!("Save world failed: {:?}", e);
        }
    }
}
