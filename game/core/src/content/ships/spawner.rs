use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::ships::types::{AIShipBehavior, AIShipController, PlayerShipController};
use crate::content::structures::block::{
    render_block_base, render_block_layer, render_block_layer_feature,
};
use crate::content::structures::registry::BlockRegistry;
use crate::content::structures::ship::{BlockData, Ship, ShipData};
use crate::content::structures::types::{BelongsToShip, BlockGroup};
use crate::content::team::Team;
use crate::{AppVar, GRID_SIZE};

pub fn spawn_player_ship(
    mut commands: Commands,
    vars: Res<AppVar>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
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

pub fn spawn_enemy_ship(
    mut commands: Commands,
    vars: Res<AppVar>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
) {
    let ship_data = if let Some(ship) = &vars.current_ship {
        ship
    } else {
        return;
    };

    spawn_ship(
        &mut commands,
        &registry,
        &assets,
        ship_data,
        Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        Team::new(1), // Enemy
    );
}

fn blocks_to_voxel_points(blocks: &[BlockData], registry: &BlockRegistry) -> Vec<Vec2> {
    let mut points = Vec::new();

    for block in blocks {
        let def = registry.get(&block.block_type).unwrap();
        let size = def.shape.size;

        // base grid position
        let base_x = block.pos.x as f32;
        let base_y = block.pos.y as f32;

        // fill rectangle
        for y in 0..size.y {
            for x in 0..size.x {
                let px = (base_x + x as f32) * GRID_SIZE;
                let py = (base_y + y as f32) * GRID_SIZE;

                points.push(Vec2::new(px, py));
            }
        }
    }

    points
}

pub fn build_ship_voxel_collider_2d(blocks: &[BlockData], registry: &BlockRegistry) -> Collider {
    let points = blocks_to_voxel_points(blocks, registry);

    return Collider::voxels_from_points(Vec2::new(GRID_SIZE, GRID_SIZE), &points);
}

pub struct ShipCalculatedStat {
    pub mass: f32,
    pub center: Vec2,
    pub collider: Collider,
}

pub fn calculate_ship_stat(
    ship_data: &Vec<BlockData>,
    registry: &BlockRegistry,
) -> ShipCalculatedStat {
    let mut mass: f32 = 0.0;
    let mut center = Vec2::ZERO;

    for block_data in ship_data {
        if let Some(def) = registry.get(&block_data.block_type) {
            mass += def.mass;

            if def.block_group == BlockGroup::Core {
                let size = def.shape.size.as_vec2();
                let offset = size * GRID_SIZE * 0.5;
                center = block_data.pos.as_vec2() * GRID_SIZE + offset;
            }
        }
    }

    return ShipCalculatedStat {
        mass,
        center: center,
        collider: build_ship_voxel_collider_2d(&ship_data, registry),
    };
}

pub fn spawn_ship(
    commands: &mut Commands,
    registry: &BlockRegistry,
    assets: &AssetServer,
    ship_data: &ShipData,
    transform: Transform,
    team: Team,
) -> Entity {
    let ShipCalculatedStat {
        mass,
        center,
        collider,
    } = calculate_ship_stat(&ship_data.blocks, registry);

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
            blocks: ship_data.blocks.clone(),
            center,
            thrust_power: 1000.0,
            turn_speed: 250.0,
            radar_range: 3000.0,
        },
        team,
    ));

    if team.is_player() {
        ship_cmd.insert(PlayerShipController);
    } else {
        ship_cmd.insert(AIShipController {
            behavior: AIShipBehavior::Idle,
            initial_behavior: AIShipBehavior::Idle,
        });
    }

    let ship_entity = ship_cmd.id();

    for block_data in &ship_data.blocks {
        if let Some(def) = registry.get(&block_data.block_type) {
            let base = render_block_base(
                commands,
                ship_entity,
                def,
                block_data.id,
                block_data.pos,
                team,
            );

            for layer in &def.layers {
                let layer_entity = render_block_layer(commands, base, layer);

                for feature in &layer.features {
                    render_block_layer_feature(commands, layer_entity, assets, def, feature, team);
                }

                commands
                    .entity(base)
                    .insert(BelongsToShip { ship: ship_entity });
            }
        }
    }

    commands.entity(ship_entity).insert((
        collider,
        CollisionGroups::new(team.get_hull_group(), team.get_hull_filter()),
        ColliderMassProperties::Mass(mass),
    ));

    return ship_entity;
}
