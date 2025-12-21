use bevy::prelude::*;

pub mod base;
pub mod shoot;
pub mod targeting;

use shoot::handle_turret_emission;
use targeting::update_turret_target;

use crate::{SceneState, content::turret::targeting::system_turret_tracking};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_turret_emission, system_turret_tracking).run_if(in_state(SceneState::Game)),
        )
        .add_systems(FixedUpdate, update_turret_target)
        .insert_resource(Time::<Fixed>::from_hz(15.0));
    }
}
