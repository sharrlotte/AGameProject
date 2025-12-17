use bevy::prelude::*;

pub mod control;
pub mod emission;
pub mod targeting;

use control::{AIFireController, FireIntent, PlayerFireController, update_player_fire_controller};
use emission::{FireStyle, TurretBarrel, TurretEmission, handle_turret_emission};
use targeting::{TurretTargeter, update_turret_target, apply_targeting_to_fire_intent};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<FireIntent>()
            .register_type::<PlayerFireController>()
            .register_type::<AIFireController>()
            .register_type::<FireStyle>()
            .register_type::<TurretBarrel>()
            .register_type::<TurretEmission>()
            .register_type::<TurretTargeter>()
            
            .add_systems(Update, (
                update_player_fire_controller,
                update_turret_target,
                apply_targeting_to_fire_intent,
                handle_turret_emission,
            ));
    }
}
