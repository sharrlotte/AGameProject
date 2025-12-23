use bevy::prelude::*;

pub mod base;
pub mod targeting;

use targeting::update_turret_target;

use crate::{
    SceneState,
    content::turret::{
        base::PotentialTargetChange,
        targeting::{system_turret_tracking, update_best_target},
    },
};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PotentialTargetChange>()
            .add_systems(
                FixedUpdate,
                system_turret_tracking.run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedUpdate,
                (update_turret_target, update_best_target).run_if(in_state(SceneState::Game)),
            );
    }
}
