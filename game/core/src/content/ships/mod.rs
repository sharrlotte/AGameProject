use bevy::prelude::*;

pub mod controller;
pub mod features;
pub mod indicators;
pub mod spawner;

use crate::AppState;
use crate::content::ships::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ships::indicators::IndicatorsPlugin;
use crate::content::ships::spawner::{spawn_enemy_ship, spawn_player_ship};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Game),
            (spawn_player_ship, spawn_enemy_ship),
        )
        .add_systems(
            Update,
            (player_ship_controller, ai_ship_controller)
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_plugins(IndicatorsPlugin);
    }
}

#[derive(Component)]
pub struct AIShipController;

#[derive(Component)]
pub struct PlayerShipController;
