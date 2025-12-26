use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    common::entity_type::EntityType, constant::LAYER_SHIP_UI, content::{
        block::{
            component::BlockData,
            resource::{BlockRegistry, ShipCalculatedStat},
        },
        team::Team,
    }
};

#[derive(Component, Debug, Default)]
pub struct Ship {
    pub id: Uuid,
    pub blocks: Vec<BlockData>,
    pub thrust_power: f32,
    pub turn_speed: f32,
    pub core_position: Vec2,
    pub center: Vec2,
    pub size: Vec2,
    pub max_range: f32,
    pub min_range: f32,
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
    pub fn spawn(
        &self,
        mut commands: &mut Commands,
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
            size,
            core_position,
            min_range,
            max_range,
        } = registry.blocks_stats(&self.blocks);

        let ship = commands
            .spawn((
                Ship {
                    id: self.id,
                    blocks: self.blocks.clone(),
                    core_position,
                    center,
                    thrust_power,
                    turn_speed,
                    size,
                    max_range,
                    min_range,
                },
                ShipState {},
                Visibility::Visible,
                EntityType::Ship,
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
                Restitution::coefficient(0.05),
                CollisionGroups::new(team.ship_group(), team.ship_filter()),
                ColliderMassProperties::Mass(mass),
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::all(),
                AIShipController::new(AIShipBehavior::Idle),
                Damping {
                    linear_damping: 0.8,
                    angular_damping: 0.8,
                },  
            ))
            .id();

        for block_data in &self.blocks {
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

        return ship;
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
