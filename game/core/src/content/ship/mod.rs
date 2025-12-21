pub mod base;
pub mod controller;
pub mod indicators;

use crate::content::block::registry::BlockRegistry;
use crate::content::ship::base::{
    AIShipController, PlayerShipController, RebuildShipCollider, Ship,
};
use crate::content::ship::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::team::Team;
use crate::math::random_vec2_step;
use crate::save::io::load_saved_schematics;
use crate::world::base::{ShipCalculatedStat, calculate_ship_stat};
use crate::{GameState, SceneState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::seq::IndexedRandom;

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
                    .run_if(in_state(SceneState::Game)),
            )
            .add_systems(OnEnter(SceneState::Home), cleanup_ships)
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
    vars: Res<GameState>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    if ships.iter().len() == 0 {
        let ship = if let Some(ship) = &vars.current_ship {
            ship
        } else {
            warn!("No ship loaded in GameState!");
            return;
        };

        let position = random_vec2_step(
            Vec2::new(-5000.0, -5000.0),
            Vec2::new(5000.0, 5000.0),
            500.0,
        );

        ship.to_ship_data().spawn(
            &mut commands,
            &registry,
            &assets,
            Team::new(0),
            Transform::from_translation(position.extend(0.0)),
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
        let schematics = load_saved_schematics();
        let Some(schematic) = schematics.choose(&mut rng) else {
            continue;
        };

        let position = random_vec2_step(
            Vec2::new(-5000.0, -5000.0),
            Vec2::new(5000.0, 5000.0),
            500.0,
        );

        schematic.to_ship_data().spawn(
            &mut commands,
            &registry,
            &assets,
            Team::new(1),
            Transform::from_translation(position.extend(0.0)),
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
        let ShipCalculatedStat {
            mass,
            center,
            collider,
            thrust_power,
            turn_speed,
        } = calculate_ship_stat(&ship.blocks, &registry);

        commands.entity(entity).remove::<Collider>().insert((
            collider,
            ColliderMassProperties::Mass(mass),
            Ship {
                id: ship.id,
                blocks: ship.blocks.clone(),
                center,
                thrust_power,
                turn_speed,
                radar_range: 3000.0,
            },
        ));

        commands.entity(entity).remove::<RebuildShipCollider>();
    }
}
