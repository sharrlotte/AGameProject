use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::ships::{AIShipBehavior, AIShipController, PlayerShipController};
use crate::content::structures::block::BlockGroup;
use crate::content::structures::registry::BlockRegistry;
use crate::content::structures::ship::{SavedShip, Ship};
use crate::content::team::Team;
use crate::{AppVar, GRID_SIZE};

pub fn spawn_player_ship(
    mut commands: Commands,
    app_var: Res<AppVar>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    query: Query<(), With<Ship>>,
) {
    if !query.is_empty() {
        return;
    }
    let ship_data = if let Some(ship) = &app_var.current_ship {
        ship
    } else {
        warn!("No ship loaded in AppVar!");
        return;
    };

    spawn_ship(
        &mut commands,
        &registry,
        &asset_server,
        ship_data,
        Transform::from_translation(Vec3::ZERO),
        Team::new(0), // Ally
        true,         // is_player
    );
}

pub fn spawn_enemy_ship(
    mut commands: Commands,
    app_var: Res<AppVar>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    query: Query<(), With<Ship>>,
) {
    if !query.is_empty() {
        return;
    }
    let ship_data = if let Some(ship) = &app_var.current_ship {
        ship
    } else {
        return;
    };

    // Spawn an enemy at (500, 500)
    spawn_ship(
        &mut commands,
        &registry,
        &asset_server,
        ship_data,
        Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        Team::new(1), // Enemy
        false,        // is_player
    );
}

pub fn spawn_ship(
    commands: &mut Commands,
    registry: &BlockRegistry,
    asset_server: &AssetServer,
    ship_data: &SavedShip,
    transform: Transform,
    team: Team,
    is_player: bool,
) -> Entity {
    let mut compound_shapes = Vec::new();
    let mut total_mass = 0.0;

    // Recalculate center based on Core block to fix legacy saves
    let mut ship_center = Vec2::ZERO;

    for saved_block in &ship_data.blocks {
        if let Some(def) = registry.get(&saved_block.block_type) {
            if def.block_group == BlockGroup::Core {
                let size = def.shape.size.as_vec2();
                let offset = size * GRID_SIZE * 0.5;
                ship_center = saved_block.pos.as_vec2() * GRID_SIZE + offset;
                break;
            }
        }
    }

    let mut ship_cmd = commands.spawn((
        transform,
        Visibility::Visible,
        RigidBody::Dynamic,
        Velocity::zero(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        ReadMassProperties::default(),
        Restitution::coefficient(0.8),
        ColliderMassProperties::Density(1.0),
        Ccd::enabled(),
        GravityScale(0.0),
        Damping {
            linear_damping: 0.8,
            angular_damping: 0.8,
        },
        Ship {
            name: ship_data.name.clone(),
            center: ship_center,
            thrust_power: 1000.0,
            turn_speed: 250.0,
            radar_range: 3000.0,
        },
        team,
    ));

    if is_player {
        ship_cmd.insert(PlayerShipController);
    } else {
        ship_cmd.insert(AIShipController {
            behavior: AIShipBehavior::Idle,
            initial_behavior: AIShipBehavior::Idle,
        });
    }

    let ship_entity = ship_cmd.id();

    for saved_block in &ship_data.blocks {
        if let Some(def) = registry.get(&saved_block.block_type) {
            let size_vec = def.shape.size.as_vec2();
            let center_offset = size_vec * GRID_SIZE * 0.5;

            let anchor_pos = Vec2::new(
                saved_block.pos.x as f32 * GRID_SIZE,
                saved_block.pos.y as f32 * GRID_SIZE,
            );

            let block_pos = anchor_pos + center_offset;
            let rotation_angle = saved_block.rotation as f32 * std::f32::consts::FRAC_PI_2;
            let size = size_vec * GRID_SIZE * 0.5;

            compound_shapes.push((
                Vect::new(block_pos.x, block_pos.y),
                rotation_angle,
                Collider::cuboid(size.x, size.y),
            ));

            total_mass += def.mass;

            let block = def.render(commands, asset_server, saved_block.pos, Some(team));

            commands.entity(ship_entity).add_child(block);
        }
    }

    if !compound_shapes.is_empty() {
        commands.entity(ship_entity).insert((
            Collider::compound(compound_shapes),
            CollisionGroups::new(team.get_hull_group(), team.get_hull_filter()),
            ColliderMassProperties::Mass(total_mass),
        ));
    } else {
        warn!("Ship has no blocks! Spawning dummy collider.");
        commands.entity(ship_entity).insert((
            Collider::ball(20.0),
            CollisionGroups::new(team.get_hull_group(), team.get_hull_filter()),
        ));
    }
    ship_entity
}
