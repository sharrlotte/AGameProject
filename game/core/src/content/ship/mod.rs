use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::seq::IndexedRandom;

pub mod base;
pub mod controller;
pub mod indicators;
pub mod spawner;

use crate::content::block::registry::BlockRegistry;
use crate::content::ship::base::{
    AIShipController, PlayerShipController, RebuildShipCollider, Ship,
};
use crate::content::ship::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::ship::spawner::{ShipCalculatedStat, calculate_ship_stat, spawn_ship};
use crate::content::team::Team;
use crate::math::random_vec2_step;
use crate::save::load_saved_schematics;
use crate::{AppState, AppVar};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComponentState>()
            .add_systems(
                Update,
                (
                    player_ship_controller,
                    ai_ship_controller,
                    check_rebuild_ship_collider,
                    check_player_ship,
                    check_enemy_ship,
                    process_input,
                )
                    .chain()
                    .run_if(in_state(AppState::Game)),
            )
            .add_systems(OnEnter(AppState::Home), cleanup_ships)
            .add_plugins(IndicatorsPlugin);
    }
}

#[derive(Resource, Default)]
pub struct ComponentState {
    cursor_world_position: Option<Vec2>,
}

fn process_input(
    mut state: ResMut<ComponentState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.0.viewport_to_world_2d(camera.1, cursor_position) {
            state.cursor_world_position = Some(world_position);
        }
    }
}

fn check_player_ship(
    mut commands: Commands,
    ships: Query<&PlayerShipController>,
    vars: Res<AppVar>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    if ships.iter().len() == 0 {
        let ship = if let Some(ship) = &vars.current_ship {
            ship
        } else {
            warn!("No ship loaded in AppVar!");
            return;
        };

        spawn_ship(
            &mut commands,
            &registry,
            &assets,
            &ship.to_ship_data(),
            Transform::from_translation(Vec3::ZERO),
            Team::new(0),
        );
    }
}

const ENEMIES: usize = 3;

fn check_enemy_ship(
    mut commands: Commands,
    ships: Query<&AIShipController>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    if ships.iter().len() > ENEMIES {
        return;
    }

    let missing = ENEMIES - ships.iter().len();
    let mut rng = rand::rng();

    for _ in 0..missing {
        let data = load_saved_schematics();
        let Some(ship) = data.choose(&mut rng) else {
            continue;
        };

        let position = random_vec2_step(
            Vec2::new(-5000.0, -5000.0),
            Vec2::new(5000.0, 5000.0),
            500.0,
        );

        spawn_ship(
            &mut commands,
            &registry,
            &assets,
            &ship.to_ship_data(),
            Transform::from_translation(position.extend(0.0)),
            Team::new(1), // Enemy
        );
    }
}

fn cleanup_ships(mut commands: Commands, ships: Query<Entity, With<Ship>>) {
    for entity in &ships {
        commands.entity(entity).despawn();
    }
}

fn check_rebuild_ship_collider(
    mut commands: Commands,
    ships: Query<(Entity, &Ship), With<RebuildShipCollider>>,
    registry: Res<BlockRegistry>,
) {
    for (entity, ship) in &ships {
        let ShipCalculatedStat { mass, collider, .. } =
            calculate_ship_stat(&ship.blocks, &registry);

        commands
            .entity(entity)
            .remove::<Collider>()
            .insert((collider, ColliderMassProperties::Mass(mass)));

        commands.entity(entity).remove::<RebuildShipCollider>();
    }
}
