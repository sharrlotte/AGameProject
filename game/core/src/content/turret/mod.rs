use bevy::prelude::*;

pub mod shoot;
pub mod targeting;
pub mod tracking;
pub mod types;

use shoot::handle_turret_emission;
use targeting::{update_player_fire_controller, update_turret_target};

pub use types::*;

use crate::AppState;
use crate::content::turret::tracking::{draw_turret_ranges, system_turret_tracking};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerFireController>()
            .register_type::<AIFireController>()
            .register_type::<ShootPattern>()
            .register_type::<TurretController>()
            .add_systems(
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
            .insert_resource(Time::<Fixed>::from_seconds(0.1));
    }
}
