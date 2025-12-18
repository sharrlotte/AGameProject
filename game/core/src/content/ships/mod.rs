use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::seq::IndexedRandom;

pub mod controller;
pub mod features;
pub mod indicators;
pub mod spawner;
pub mod types;

use crate::content::ships::controller::{ai_ship_controller, player_ship_controller};
use crate::content::ships::indicators::IndicatorsPlugin;
use crate::content::ships::spawner::{ShipCalculatedStat, calculate_ship_stat, spawn_ship};
use crate::content::structures::registry::BlockRegistry;
use crate::content::structures::ship::Ship;
use crate::content::team::Team;
use crate::math::random_vec2_step;
use crate::save::load_saved_ships;
use crate::{AppState, AppVar};

pub use types::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_ship_controller,
                ai_ship_controller,
                check_rebuild_ship_collider,
                check_player_ship,
                check_enemy_ship,
            )
                .chain()
                .run_if(in_state(AppState::Game)),
        )
        .add_systems(OnEnter(AppState::Home), cleanup_ships)
        .add_plugins(IndicatorsPlugin);
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
        let ship_data = if let Some(ship) = &vars.current_ship {
            ship
        } else {
            warn!("No ship loaded in AppVar!");
            return;
        };

        spawn_ship(
            &mut commands,
            &registry,
            &assets,
            ship_data,
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
    if ships.iter().len() <= ENEMIES {
        let missing = ENEMIES - ships.iter().len();
        let mut rng = rand::rng();
        for _ in 0..missing {
            let data = load_saved_ships();
            let Some(ship_data) = data.choose(&mut rng) else {
                continue;
            };

            let pos = random_vec2_step(
                Vec2::new(-5000.0, -5000.0),
                Vec2::new(5000.0, 5000.0),
                500.0,
            );

            spawn_ship(
                &mut commands,
                &registry,
                &assets,
                ship_data,
                Transform::from_translation(pos.extend(0.0)),
                Team::new(1), // Enemy
            );
        }
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

        info!("Rebuild ship collider for entity {:?}", entity);
    }
}
