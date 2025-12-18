use bevy::prelude::*;

pub mod controller;
pub mod features;
pub mod indicators;
pub mod spawner;
pub mod types;

use crate::content::ships::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ships::indicators::IndicatorsPlugin;
use crate::content::ships::spawner::{spawn_enemy_ship, spawn_player_ship};
use crate::content::structures::registry::BlockRegistry;
use crate::content::structures::ship::Ship;
use crate::{AppState, AppVar};

pub use types::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Game),
            (check_player_ship, check_enemy_ship),
        )
        .add_systems(
            Update,
            (player_ship_controller, ai_ship_controller)
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(OnEnter(AppState::Home), cleanup_ships)
        .add_plugins(IndicatorsPlugin);
    }
}

fn check_player_ship(
    commands: Commands,
    ships: Query<&PlayerShipController>,
    vars: Res<AppVar>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    if ships.iter().len() == 0 {
        spawn_player_ship(commands, vars, registry, assets);
    }
}

fn check_enemy_ship(
    commands: Commands,
    ships: Query<&AIShipController>,
    vars: Res<AppVar>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    if ships.iter().len() == 0 {
        spawn_enemy_ship(commands, vars, registry, assets);
    }
}

fn cleanup_ships(mut commands: Commands, ships: Query<Entity, With<Ship>>) {
    for entity in &ships {
        commands.entity(entity).despawn();
    }
}
