use std::time::Duration;

use crate::SceneState;
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::ship::system::{
    check_rebuild_ship_collider, check_ship, cleanup_ships, collect_drop, control_ship,
    player_control_ship, update_ship_behavior, update_ship_ui,
};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_ship_ui.run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            PostUpdate,
            update_ship_behavior.run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            (
                control_ship,
                check_rebuild_ship_collider,
                player_control_ship,
            )
                .run_if(in_state(SceneState::Game)),
        )
        .add_systems(
            FixedUpdate,
            check_ship.run_if(on_timer(Duration::from_millis(100)).and(in_state(SceneState::Game))),
        )
        .add_systems(FixedUpdate, collect_drop.run_if(in_state(SceneState::Game)))
        .add_systems(OnEnter(SceneState::Home), cleanup_ships)
        .add_plugins(IndicatorsPlugin);
    }
}
