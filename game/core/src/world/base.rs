use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    components::entity_type::EntityType,
    constant::GRID_SIZE,
    content::{
        block::{
            base::{
                Block, BlockComponent, BlockDef, BlockGroup, BlockState, BlockType,
                BurstShootPatternState, ContinuousShootPatternState, GridPosition, Health,
                ShipIdComponent, ShootPattern, SingleShootPatternState, Thruster,
                ThrusterComponent,
            },
            registry::BlockRegistry,
        },
        ship::base::{AIShipBehavior, AIShipController, PlayerShipController, Ship, ShipState},
        team::Team,
        turret::base::{Turret, TurretState},
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
    pub thrust_power: f32,
    pub turn_speed: f32,
}

pub fn calculate_ship_stat(
    blocks: &Vec<BlockData>,
    registry: &BlockRegistry,
) -> ShipCalculatedStat {
    let mut mass: f32 = 0.0;
    let mut center = Vec2::ZERO;
    let mut thrust_power: f32 = 1000.0;
    let mut turn_speed: f32 = 250.0;

    for block in blocks {
        let Some(def) = registry.get(&block.block_type) else {
            warn!("Block type {:?} not found in registry", block.block_type);
            continue;
        };

        mass += def.mass;

        if def.block_group == BlockGroup::Core {
            let size = def.shape.size.as_vec2();
            let offset = size * GRID_SIZE * 0.5;
            center = block.position.as_vec2() * GRID_SIZE + offset;
        }

        for component in &def.components {
            match component {
                BlockComponent::Thruster(ThrusterComponent { force, torque }) => {
                    thrust_power += force;
                    turn_speed += torque;
                }
                _ => {}
            }
        }
    }

    return ShipCalculatedStat {
        thrust_power,
        turn_speed,
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
        team: Team,
        transform: Transform,
    ) -> Entity {
        let ShipCalculatedStat {
            mass,
            center,
            collider,
            thrust_power,
            turn_speed,
        } = calculate_ship_stat(&self.blocks, registry);

        let ship = commands
            .spawn((
                transform,
                Visibility::Visible,
                Ship {
                    id: self.id,
                    blocks: self.blocks.clone(),
                    center,
                    thrust_power,
                    turn_speed,
                    radar_range: 3000.0,
                },
                ShipState {},
                EntityType::Ship,
                team,
            ))
            .insert((
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                ExternalImpulse::default(),
                ReadMassProperties::default(),
                Restitution::coefficient(0.05),
                ColliderMassProperties::Density(1.0),
                GravityScale(0.0),
                Damping {
                    linear_damping: 0.7,
                    angular_damping: 0.9,
                },
            ))
            .id();

        if team.is_player() {
            commands.entity(ship).insert(PlayerShipController);
        } else {
            commands
                .entity(ship)
                .insert(AIShipController::new(AIShipBehavior::Idle));
        }

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

                commands.entity(base).insert(ShipIdComponent { id: ship });

                for component in &def.components {
                    render_block_component(commands, base, def, component, team);
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
            ShipIdComponent { id: ship },
            EntityType::Block,
        ))
        .id();

    commands.entity(ship).add_child(child);

    return child;
}

pub fn render_block_component(
    commands: &mut Commands,
    parent: Entity,
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
            commands.entity(child).insert((
                Turret {
                    strategy: f.strategy.clone(),
                    range: f.range,
                    turn_speed: f.turn_speed,
                    barrels: f.barrels.clone(),
                    shooting_pattern: f.shooting_pattern.clone(),
                },
                TurretState {
                    potential_targets: HashSet::default(),
                    active: false,
                    target: None,
                    shoot_at: None,
                },
                Sensor,
                Collider::ball(f.range),
                CollisionGroups::new(team.get_radar_group(), team.get_radar_filter()),
                ActiveEvents::COLLISION_EVENTS,
                ColliderMassProperties::Mass(0.0),
                Sprite {
                    image: f.image.clone(),
                    custom_size: Some(get_block_size(def)),
                    ..default()
                },
                Transform::from_translation(f.offset.extend(f.z)),
            ));

            match f.shooting_pattern.clone() {
                ShootPattern::Single(s) => {
                    commands
                        .entity(child)
                        .insert((s, SingleShootPatternState { last_fired_at: 0.0 }));
                }
                ShootPattern::Burst(s) => {
                    commands.entity(child).insert((
                        s,
                        BurstShootPatternState {
                            last_fired_at: 0.0,
                            current_barrel: 0,
                            last_barrel_fired_at: 0.0,
                            current_amount: 0,
                            is_bursting: false,
                        },
                    ));
                }
                ShootPattern::Continuous(s) => {
                    commands
                        .entity(child)
                        .insert((s, ContinuousShootPatternState {}));
                }
            }
        }
        BlockComponent::Sprite(f) => {
            commands.entity(child).insert((
                Sprite {
                    image: f.image.clone(),
                    custom_size: Some(get_block_size(def)),
                    ..default()
                },
                Transform::from_translation(f.offset.extend(f.z)),
            ));
        }
        BlockComponent::Thruster(ThrusterComponent { force, torque }) => {
            commands.entity(child).insert((Thruster {
                force: *force,
                torque: *torque,
            },));
        }
    };

    return child;
}
