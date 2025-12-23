use std::time::Duration;

use crate::SceneState;
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::ship::system::{
    ai_ship_controller, check_enemy_ship, check_player_ship, check_rebuild_ship_collider,
    cleanup_ships, player_ship_controller,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            player_ship_controller.run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            (ai_ship_controller, check_rebuild_ship_collider).run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            (check_player_ship, check_enemy_ship)
                .run_if(on_timer(Duration::from_secs(1)).and(in_state(SceneState::Game))),
        )
        .add_systems(OnEnter(SceneState::Home), cleanup_ships)
        .add_plugins(IndicatorsPlugin);
    }
}
