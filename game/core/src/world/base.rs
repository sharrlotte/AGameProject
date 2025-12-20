use bevy::{
    asset::AssetServer,
    camera::visibility::Visibility,
    ecs::{entity::Entity, system::Commands},
    math::{IVec2, Vec2},
};
use bevy_rapier2d::prelude::{
    Ccd, Collider, ColliderMassProperties, CollisionGroups, Damping, ExternalForce,
    ExternalImpulse, GravityScale, ReadMassProperties, Restitution, RigidBody, Velocity,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    GRID_SIZE,
    content::{
        block::{
            base::{BlockGroup, BlockType, ShipIdComponent},
            registry::BlockRegistry,
            spawner::{render_block_base, render_block_component},
        },
        ship::base::{AIShipBehavior, AIShipController, PlayerShipController, Ship, ShipState},
        team::Team,
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
    ) -> Entity {
        let ShipCalculatedStat {
            mass,
            center,
            collider,
        } = calculate_ship_stat(&self.blocks, registry);

        let mut ship_cmd = commands.spawn((
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
                linear_damping: 0.9,
                angular_damping: 0.99,
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
