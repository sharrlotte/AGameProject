use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

pub mod base;
pub mod targeting;

use targeting::update_turret_target;

use crate::{SceneState, content::turret::targeting::system_turret_tracking};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            system_turret_tracking.run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            update_turret_target
                .run_if(on_timer(Duration::from_millis(100)).and(in_state(SceneState::Game))),
        );
    }
}
