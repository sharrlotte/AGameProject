use crate::content::structure::block::{BlockShape, BlockType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enum representing distinct features/behaviors a block can have.
/// This allows us to extend block functionality without growing the base struct.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum BlockFeature {
    /// Provides thrust force.
    Thruster { force: f32 },
    /// Generates power.
    PowerProducer { output: f32 },
    /// Consumes power to function.
    PowerConsumer { required: f32 },
    /// Stores power.
    PowerStorage { capacity: f32 },
    /// Can deal damage.
    Weapon {
        damage: f32,
        cooldown: f32,
        range: f32,
    },
}

/// A definition struct that holds the static data for a block type.
/// This allows us to define blocks once and look them up later.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockDefinition {
    pub name: String,
    pub description: String,
    pub mass: f32,
    pub max_health: f32,
    /// Replaced `size` with `shape` to support complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    /// A list of features this block possesses.
    pub features: Vec<BlockFeature>,
    pub texture_paths: Vec<String>,
}

/// A global resource to register and retrieve block definitions.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    map: HashMap<BlockType, BlockDefinition>,
}

impl BlockRegistry {
    pub fn register(&mut self, block_type: BlockType, def: BlockDefinition) {
        self.map.insert(block_type, def);
    }

    pub fn get(&self, block_type: &BlockType) -> Option<&BlockDefinition> {
        self.map.get(block_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&BlockType, &BlockDefinition)> {
        self.map.iter()
    }
}

/// Helper system to register default blocks (can be expanded to load from files later).
pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>) {
    // 1. Basic Hull
    registry.register(
        BlockType::Hull,
        BlockDefinition {
            name: "Standard Hull".into(),
            description: "Basic structural block.".into(),
            mass: 10.0,
            max_health: 100.0,
            shape: BlockShape::default(),
            cost: 5,
            features: vec![],
            texture_paths: vec!["textures/blocks/surge-wall.png".into()],
        },
    );

    // 2. Small Thruster
    registry.register(
        BlockType::ThrusterSmall,
        BlockDefinition {
            name: "Small Ion Thruster".into(),
            description: "Provides weak thrust.".into(),
            mass: 25.0,
            max_health: 50.0,
            shape: BlockShape::default(),
            cost: 50,
            features: vec![
                BlockFeature::Thruster { force: 100.0 },
                BlockFeature::PowerConsumer { required: 10.0 },
            ],
            texture_paths: vec!["textures/blocks/surge-wall.png".into()],
        },
    );

    // 3. Small Reactor
    registry.register(
        BlockType::ReactorSmall,
        BlockDefinition {
            name: "Micro Fusion Reactor".into(),
            description: "Generates power.".into(),
            mass: 100.0,
            max_health: 80.0,
            shape: BlockShape::default(),
            cost: 200,
            features: vec![BlockFeature::PowerProducer { output: 50.0 }],
            texture_paths: vec!["textures/blocks/surge-wall.png".into()],
        },
    );

    // 4. Cockpit
    registry.register(
        BlockType::Cockpit,
        BlockDefinition {
            name: "Command Cockpit".into(),
            description: "Pilot seat.".into(),
            mass: 50.0,
            max_health: 60.0,
            shape: BlockShape::default(),
            cost: 150,
            features: vec![BlockFeature::PowerConsumer { required: 5.0 }],
            texture_paths: vec!["textures/blocks/surge-wall.png".into()],
        },
    );
    registry.register(
        BlockType::Core,
        BlockDefinition {
            name: "Core".into(),
            description: "Core".into(),
            mass: 50.0,
            max_health: 60.0,
            shape: BlockShape::default(),
            cost: 150,
            features: vec![BlockFeature::PowerConsumer { required: 5.0 }],
            texture_paths: vec!["textures/blocks/core-nucleus.png".into(), "textures/blocks/core-nucleus-team.png".into()],
        },
    );
}
