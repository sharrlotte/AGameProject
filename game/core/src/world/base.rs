use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    GRID_SIZE,
    content::{
        block::{
            base::{Block, BlockComponent, BlockDef, BlockGroup, BlockState, BlockType, GridPosition, Health, ShipIdComponent, ShootPattern},
            registry::BlockRegistry,
        },
        ship::base::{AIShipBehavior, AIShipController, PlayerShipController, Ship, ShipState},
        team::Team, turret::base::{AIFireController, PlayerFireController, Turret, TurretState},
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShipData {
    pub id: Uuid,
    pub blocks: Vec<BlockData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

fn blocks_to_voxel_points(blocks: &[BlockData], registry: &BlockRegistry) -> Vec<Vec2> {
    let mut points = Vec::new();

    for block in blocks {
        let def = registry.get(&block.block_type).unwrap();
        let size = def.shape.size;

        // base grid position
        let base_x = block.position.x as f32;
        let base_y = block.position.y as f32;

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
    blocks: &Vec<BlockData>,
    registry: &BlockRegistry,
) -> ShipCalculatedStat {
    let mut mass: f32 = 0.0;
    let mut center = Vec2::ZERO;

    for block in blocks {
        if let Some(def) = registry.get(&block.block_type) {
            mass += def.mass;

            if def.block_group == BlockGroup::Core {
                let size = def.shape.size.as_vec2();
                let offset = size * GRID_SIZE * 0.5;
                center = block.position.as_vec2() * GRID_SIZE + offset;
            }
        }
    }

    return ShipCalculatedStat {
        mass,
        center: center,
        collider: build_ship_voxel_collider_2d(&blocks, registry),
    };
}

impl ShipData {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        assets: &AssetServer,
        team: Team,
        transform: Transform,
    ) -> Entity {
        let ShipCalculatedStat {
            mass,
            center,
            collider,
        } = calculate_ship_stat(&self.blocks, registry);

        let mut ship_cmd = commands.spawn((
            transform,
            Visibility::Visible,
            RigidBody::Dynamic,
            Velocity::zero(),
            ExternalForce::default(),
            ExternalImpulse::default(),
            ReadMassProperties::default(),
            Restitution::coefficient(0.2),
            Ccd::enabled(),
            ColliderMassProperties::Density(1.0),
            GravityScale(0.0),
            Damping {
                linear_damping: 0.7,
                angular_damping: 0.9,
            },
            Ship {
                id: self.id,
                blocks: self.blocks.clone(),
                center,
                thrust_power: 1000.0,
                turn_speed: 250.0,
                radar_range: 3000.0,
            },
            ShipState {},
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

        let ship = ship_cmd.id();

        for block_data in &self.blocks {
            if let Some(def) = registry.get(&block_data.block_type) {
                let base = render_block_base(
                    commands,
                    ship,
                    def,
                    block_data.id,
                    block_data.position,
                    team,
                );

                commands.entity(base).insert(ShipIdComponent { ship });

                for component in &def.components {
                    render_block_component(commands, base, assets, def, component, team);
                }
            }
        }

        commands.entity(ship).insert((
            collider,
            CollisionGroups::new(team.get_hull_group(), team.get_hull_filter()),
            ColliderMassProperties::Mass(mass),
        ));

        return ship;
    }
}

pub fn get_block_size(def: &BlockDef) -> Vec2 {
    let size = def.shape.size.as_vec2();
    return size * GRID_SIZE;
}

pub fn get_block_half_size(def: &BlockDef) -> Vec2 {
    return get_block_size(def) * 0.5;
}

pub fn get_block_transform(position: IVec2, half_size: Vec2) -> Transform {
    let anchor_position = Vec3::new(
        position.x as f32 * GRID_SIZE,
        position.y as f32 * GRID_SIZE,
        0.0,
    );

    let position = anchor_position + half_size.extend(0.0);

    return Transform::from_translation(position);
}

pub fn render_block_base(
    commands: &mut Commands,
    ship: Entity,
    def: &BlockDef,
    id: Uuid,
    position: IVec2,
    team: Team,
) -> Entity {
    let half_size = get_block_half_size(def);
    let transform = get_block_transform(position, half_size);

    let child = commands
        .spawn((
            transform,
            def.block_group,
            team,
            Visibility::Visible,
            GridPosition(position),
            Collider::cuboid(half_size.x, half_size.y),
            CollisionGroups::new(team.get_block_group(), team.get_block_filter()),
            ColliderMassProperties::Mass(0.0),
            Block {
                id,
                rotation: 0,
                block_type: def.block_type,
                position,
            },
            BlockState {
                block_type: def.block_type,
            },
            Health {
                health: def.health,
                max_health: def.health,
            },
            ShipIdComponent { ship },
        ))
        .id();

    commands.entity(ship).add_child(child);

    return child;
}

pub fn render_block_component(
    commands: &mut Commands,
    parent: Entity,
    assets: &AssetServer,
    def: &BlockDef,
    component: &BlockComponent,
    team: Team,
) -> Entity {
    let child = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
            component.clone(),
        ))
        .id();

    commands.entity(parent).add_child(child);
    commands.entity(child).insert(team);

    match component {
        BlockComponent::Recoil(_recoil_component) => todo!(),
        BlockComponent::HeatEmitter(_heat_emitter_component) => todo!(),
        BlockComponent::EnergyConsumer(_energy_consumer_component) => todo!(),
        BlockComponent::Turret(f) => {
            if team.is_player() {
                commands.entity(child).insert(PlayerFireController);
            } else {
                commands.entity(child).insert(AIFireController);
            }

            commands.entity(child).insert((
                Turret {
                    strategy: f.strategy.clone(),
                    range: f.range,
                    turn_speed: f.turn_speed,
                    fire_all_at_once: f.fire_all_at_once,
                    barrels: f.barrels.clone(),
                    shooting_pattern: f.shooting_pattern.clone(),
                },
                TurretState {
                    potential_targets: HashSet::default(),
                    active: false,
                    target: None,
                    shoot_at: None,
                },
                f.shooting_pattern.clone(),
                ShootPattern::new(&f.shooting_pattern),
                Sensor,
                Collider::ball(f.range),
                CollisionGroups::new(team.get_radar_group(), team.get_radar_filter()),
                ColliderMassProperties::Mass(0.0),
                ActiveEvents::COLLISION_EVENTS,
                Sprite {
                    image: assets.load(&f.sprite),
                    custom_size: Some(get_block_size(def)),
                    color: Color::srgb(1.005, 1.005, 1.005),
                    ..default()
                },
                Transform::from_translation(f.offset.extend(f.z)),
            ));
        }
        BlockComponent::Sprite(f) => {
            commands.entity(child).insert((
                Sprite {
                    image: assets.load(&f.sprite),
                    custom_size: Some(get_block_size(def)),
                    color: Color::srgb(1.005, 1.005, 1.005),
                    ..default()
                },
                Transform::from_translation(f.offset.extend(f.z)),
            ));
        }
    };

    return child;
}
