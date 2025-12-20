use bevy::prelude::*;

pub mod base;
pub mod shoot;
pub mod targeting;
pub mod tracking;

use shoot::handle_turret_emission;
use targeting::{update_player_fire_controller, update_turret_target};

use crate::AppState;
use crate::content::turret::tracking::{draw_turret_ranges, system_turret_tracking};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_player_fire_controller,
                handle_turret_emission,
                system_turret_tracking,
                draw_turret_ranges,
            )
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(FixedUpdate, update_turret_target)
        .insert_resource(Time::<Fixed>::from_hz(15.0));
    }
}
