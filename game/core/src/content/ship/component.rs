use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(debug_assertions)]
use crate::constant::LAYER_SHIP_UI;
use crate::config::GameConfigValue;
use crate::content::block::{BlockData, BlockRegistry, ShipCalculatedStat};
use crate::content::ship::grid::ShipGrid;
use crate::content::ship::ai::{BehaviorIdle, BehaviorTimer};
use crate::content::selectable::Selectable;
use crate::content::ship::stat::ComputedShipStats;
use crate::content::team::Team;
use crate::resource::{SceneEntity, SceneState};

#[derive(Component, Debug, Default, Clone)]
pub struct Ship {
    pub id: Uuid,
    pub core_position: Vec2,
    pub max_range: f32,
    pub min_range: f32,
    pub mass: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct BoundingBox {
    pub size: Vec2,
}

#[derive(Component, Debug, Default, Clone)]
pub struct ShipFragment {
    pub blocks: Vec<BlockData>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Blocks {
    pub blocks: Vec<BlockData>,
}

#[derive(Component, Debug, Default, Reflect, Clone, Serialize, Deserialize)]
pub struct ShipState;


#[derive(Serialize, Deserialize, Debug)]
pub struct ShipData {
    pub id: Uuid,
    pub blocks: Vec<BlockData>,
}

impl ShipData {
    pub fn new(&self) -> ShipData {
        Self {
            id: Uuid::now_v7(),
            blocks: self.blocks.iter().map(|f| f.new()).collect(),
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        team: Team,
        transform: Transform,
        config: &GameConfigValue,
    ) -> Option<(Entity, Vec<Entity>)> {
        let mut grid = ShipGrid::new();
        grid.rebuild(&self.blocks, registry);
        let ShipCalculatedStat { ship, .. } = registry.blocks_stats(&self.blocks, &grid);

        let Some(data) = ship else {
            return None;
        };

        let shipc = data.data.clone();
        let Ship { mass, .. } = data.data;
        let collider = registry.build_ship_voxel_collider_2d(&grid);

        let ship = commands
            .spawn((
                shipc,
                ShipState {},
                ComputedShipStats::default(),
                Visibility::Visible,
                Selectable,
                Blocks {
                    blocks: data.blocks.clone(),
                },
                grid,
                BoundingBox { size: data.size },
                SceneEntity(SceneState::Game),
                transform,
                team,
            ))
            .insert((
                collider,
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                ExternalImpulse::default(),
                ReadMassProperties::default(),
                Restitution::coefficient(0.005),
                CollisionGroups::new(team.ship_group(), team.ship_filter()),
                ColliderMassProperties::Mass(mass),
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::all(),
                BehaviorTimer::new(config.0.ai_think_rate_hz),
                BehaviorIdle,
                Damping {
                    linear_damping: config.0.linear_damping,
                    angular_damping: config.0.angular_damping,
                },
            ))
            .id();

        let mut block_entities = Vec::new();
        for block_data in &data.blocks {
            if let Some(def) = registry.get(&block_data.block_type) {
                block_entities.push(block_data.spawn(commands, def, ship, team, config));
            }
        }

        #[cfg(debug_assertions)]
        {
            let ui_root = commands
                .spawn((
                    ShipUIRoot { ship },
                    Visibility::Visible,
                    Transform::from_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
                ))
                .id();

            commands.entity(ship).add_child(ui_root);
        }

        return Some((ship, block_entities));
    }
}

#[cfg(debug_assertions)]
#[derive(Component, Debug)]
pub struct ShipUIRoot {
    pub ship: Entity,
}

#[derive(Component, Debug)]
pub enum ShipBehavior {
    Approach { position: Vec2, radius: f32 },
    GoAround { position: Vec2, radius: f32 },
}
