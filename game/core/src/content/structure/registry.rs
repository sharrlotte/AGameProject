use crate::content::structure::block::{BlockDefinition, BlockFeature, BlockShape, BlockType};
use bevy::prelude::*;
use std::collections::HashMap;

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

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>) {
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
            texture_paths: vec!["textures/blocks/surge-wall-large.png".into()],
        },
    );

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
            texture_paths: vec!["textures/blocks/surge-wall-large.png".into()],
        },
    );

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
            texture_paths: vec!["textures/blocks/surge-wall-large.png".into()],
        },
    );

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
            texture_paths: vec!["textures/blocks/surge-wall-large.png".into()],
        },
    );

    registry.register(
        BlockType::Core,
        BlockDefinition {
            name: "Core".into(),
            description: "Core".into(),
            mass: 50.0,
            max_health: 60.0,
            shape: BlockShape::filled(3, 3),
            cost: 150,
            features: vec![BlockFeature::PowerConsumer { required: 5.0 }],
            texture_paths: vec![
                "textures/blocks/core-nucleus.png".into(),
                "textures/blocks/core-nucleus-team.png".into(),
            ],
        },
    );

    registry.register(
        BlockType::LazerTurret,
        BlockDefinition {
            name: "Lazer Turret".into(),
            description: "Core".into(),
            mass: 50.0,
            max_health: 60.0,
            shape: BlockShape::filled(1, 1),
            cost: 150,
            features: vec![BlockFeature::PowerConsumer { required: 5.0 }],
            texture_paths: vec![
                "textures/blocks/block-2.png".into(),
                "textures/blocks/lancer.png".into(),
            ],
        },
    );
}
