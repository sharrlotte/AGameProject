pub mod base;
pub mod controller;
pub mod indicators;

use std::time::Duration;

use crate::components::death::Death;
use crate::content::block::BlockDestroyedEvent;
use crate::content::block::base::BlockType;
use crate::content::block::registry::BlockRegistry;
use crate::content::ship::base::{AIShipController, PlayerShipController, Ship};
use crate::content::ship::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::team::Team;
use crate::save::io::load_saved_schematics;
use crate::utils::random::random_vec2_step;
use crate::world::base::{ShipCalculatedStat, calculate_ship_stat};
use crate::{GameState, SceneState};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::Rng;
use rand::seq::IndexedRandom;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComponentState>()
            .add_systems(
                Update,
                (process_input, player_ship_controller).run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedUpdate,
                (ai_ship_controller, check_rebuild_ship_collider)
                    .run_if(in_state(SceneState::Game)),
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
) {
    if ships.iter().len() == 0 {
        let ship = if let Some(ship) = &vars.current_ship {
            ship
        } else {
            warn!("No ship loaded in GameState!");
            return;
        };

        let position = random_vec2_step(
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(10000.0, 10000.0),
            5000.0,
        );

        ship.to_ship_data().spawn(
            &mut commands,
            &registry,
            Team::new(0),
            Transform::from_translation(position.extend(0.0)),
        );
    }
}

const ENEMIES: usize = 20;

fn check_enemy_ship(
    mut commands: Commands,
    ships: Query<&AIShipController>,
    registry: Res<BlockRegistry>,
) {
    if ships.iter().len() > ENEMIES {
        return;
    }

    let missing = ENEMIES - ships.iter().len();
    let mut rng = rand::rng();

    for _ in 0..missing {
        let team_id = rng.random_range(1..10);
        let schematics = load_saved_schematics();
        let Some(schematic) = schematics.choose(&mut rng) else {
            continue;
        };

        let position = random_vec2_step(
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(10000.0, 10000.0),
            500.0,
        );

        schematic.to_ship_data().spawn(
            &mut commands,
            &registry,
            Team::new(team_id),
            Transform::from_translation(position.extend(0.0)),
        );
    }
}

fn cleanup_ships(mut commands: Commands, ships: Query<Entity, With<Ship>>) {
    for entity in &ships {
        commands.entity(entity).insert(Death);
    }
}

fn check_rebuild_ship_collider(
    mut commands: Commands,
    mut events: MessageReader<BlockDestroyedEvent>,
    mut ships: Query<(Entity, &mut Ship, &Team)>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((entity, mut ship, _)) = ships.get_mut(event.ship_id) else {
            continue;
        };

        if let Some(block) = registry.get(&event.block) {
            if block.block_type == BlockType::Core {
                commands.entity(entity).insert(Death);
                continue;
            }
        }

        ship.blocks.retain_mut(|f| f.id != event.block_id);

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
    }
}
