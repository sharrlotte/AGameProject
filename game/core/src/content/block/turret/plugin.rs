use crate::{
    SceneState,
    content::block::turret::system::{turret_tracking, update_turret_target},
};

use bevy::prelude::*;
pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (turret_tracking, update_turret_target).run_if(in_state(SceneState::Game)),
        );
    }
}
