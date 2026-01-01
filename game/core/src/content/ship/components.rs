use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::constant::{ANGULAR_DAMPING, LAYER_SHIP_UI, LINEAR_DAMPING};
use crate::content::block::{BlockData, BlockRegistry, ShipCalculatedStat};
use crate::content::entity::EntityType;
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
    pub mass: f32,
    pub blocks: Vec<BlockData>,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Blocks {
    pub blocks: Vec<BlockData>,
}

#[derive(Component, Debug, Default, Reflect, Clone, Serialize, Deserialize)]
pub struct ShipState;

#[derive(Component)]
pub struct AIShipController {
    last_behavior: AIShipBehavior,
    behavior: AIShipBehavior,
    initial_behavior: AIShipBehavior,
    pub timer: Timer,
}

impl AIShipController {
    pub fn new(initial_behavior: AIShipBehavior) -> Self {
        Self {
            last_behavior: initial_behavior,
            behavior: initial_behavior,
            initial_behavior,
            timer: Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating),
        }
    }

    pub fn next(&mut self, behavior: AIShipBehavior) {
        self.last_behavior = self.behavior;
        self.behavior = behavior;
    }

    pub fn previous(&mut self) {
        self.behavior = self.last_behavior;
        self.last_behavior = self.initial_behavior;
    }

    pub fn current(&self) -> AIShipBehavior {
        self.behavior
    }

    pub fn last(&self) -> AIShipBehavior {
        self.last_behavior
    }

    pub fn reset(&mut self) {
        self.behavior = self.initial_behavior;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AIShipBehavior {
    Idle,
    Wander,
    Attack { target_position: Vec2 },
    Patrol { around_position: Vec2, radius: f32 },
    Chase { target: Entity },
    Flee { from: Entity },
    Guard { target: Entity, radius: f32 },
    Follow { target: Entity, radius: f32 },
    Kamikaze { target: Entity },
    Regroup,
    Kiting,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct ShipGrid {
    /// Maps (x, y) -> Entity ID
    #[reflect(ignore)]
    pub blocks: HashMap<IVec2, Entity>,
}

impl ShipGrid {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, position: IVec2, entity: Entity) {
        self.blocks.insert(position, entity);
    }

    pub fn get(&self, position: IVec2) -> Option<&Entity> {
        self.blocks.get(&position)
    }

    pub fn remove(&mut self, position: IVec2) -> Option<Entity> {
        self.blocks.remove(&position)
    }
}

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
        mut commands: &mut Commands,
        registry: &BlockRegistry,
        team: Team,
        transform: Transform,
    ) -> Option<Entity> {
        let ShipCalculatedStat { ship, .. } = registry.blocks_stats(&self.blocks);

        let Some(data) = ship else {
            return None;
        };

        let shipc = data.data.clone();
        let Ship { mass, .. } = data.data;
        let collider = registry.build_ship_voxel_collider_2d(&data.blocks);

        let ship = commands
            .spawn((
                shipc,
                ShipState {},
                ComputedShipStats::default(),
                Visibility::Visible,
                EntityType::Ship,
                Selectable,
                Blocks {
                    blocks: data.blocks.clone(),
                },
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
                AIShipController::new(AIShipBehavior::Idle),
                Damping {
                    linear_damping: LINEAR_DAMPING,
                    angular_damping: ANGULAR_DAMPING,
                },
            ))
            .id();

        for block_data in &data.blocks {
            if let Some(def) = registry.get(&block_data.block_type) {
                block_data.spawn(&mut commands, def, ship, team);
            }
        }

        let ui_root = commands
            .spawn((
                ShipUIRoot { ship },
                Visibility::Visible,
                Transform::from_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
            ))
            .id();

        commands.entity(ship).add_child(ui_root);

        return Some(ship);
    }
}

#[derive(Component, Debug)]
pub struct ShipUIRoot {
    pub ship: Entity,
}

#[derive(Component, Debug)]
pub enum ShipBehavior {
    Approach { position: Vec2, radius: f32 },
    GoAround { position: Vec2, radius: f32 },
}
