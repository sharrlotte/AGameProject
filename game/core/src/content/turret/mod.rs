use bevy::prelude::*;

pub mod base;
pub mod targeting;

use crate::{
    SceneState,
    content::turret::targeting::{system_turret_tracking, update_turret_target},
};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (system_turret_tracking, update_turret_target).run_if(in_state(SceneState::Game)),
        );
    }
}
