use bevy::prelude::*;
use std::collections::HashMap;

use crate::content::block::base::{
    BlockComponent, BlockDef, BlockGroup, BlockShape, BlockType, ShootPattern, ShootingProjectile,
    SpriteComponent, TargetingStrategy, TurretComponent,
};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    map: HashMap<BlockType, BlockDef>,
}

impl BlockRegistry {
    fn register(&mut self, block_type: BlockType, def: BlockDef) {
        if self.map.contains_key(&block_type) {
            warn!("Block type {:#?} is already registered.", block_type);
            return;
        }

        self.map.insert(block_type, def);
    }

    pub fn get(&self, block_type: &BlockType) -> Option<&BlockDef> {
        self.map.get(block_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&BlockType, &BlockDef)> {
        self.map.iter()
    }
}

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>) {
    registry.register(
        BlockType::Hull,
        BlockDef {
            name: "Standard Hull".into(),
            description: "Basic structural block.".into(),
            block_type: BlockType::Hull,
            block_group: BlockGroup::Base,
            mass: 10.0,
            health: 10000.0,
            shape: BlockShape::default(),
            cost: 5,
            components: vec![BlockComponent::Sprite(SpriteComponent {
                sprite: "textures/blocks/surge-wall-large.png".into(),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            })],
        },
    );

    registry.register(
        BlockType::ThrusterSmall,
        BlockDef {
            name: "Small Ion Thruster".into(),
            description: "Provides weak thrust.".into(),
            block_type: BlockType::ThrusterSmall,
            block_group: BlockGroup::Base,

            mass: 25.0,
            health: 5000.0,
            shape: BlockShape::default(),
            cost: 50,
            components: vec![BlockComponent::Sprite(SpriteComponent {
                sprite: "textures/blocks/surge-wall-large.png".into(),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            })],
        },
    );

    registry.register(
        BlockType::ReactorSmall,
        BlockDef {
            name: "Micro Fusion Reactor".into(),
            description: "Generates power.".into(),
            block_type: BlockType::ReactorSmall,
            block_group: BlockGroup::Base,

            mass: 100.0,
            health: 800.0,
            shape: BlockShape::default(),
            cost: 200,
            components: vec![BlockComponent::Sprite(SpriteComponent {
                sprite: "textures/blocks/surge-wall-large.png".into(),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            })],
        },
    );

    registry.register(
        BlockType::Cockpit,
        BlockDef {
            name: "Command Cockpit".into(),
            description: "Pilot seat.".into(),
            block_type: BlockType::Cockpit,
            block_group: BlockGroup::Base,

            mass: 50.0,
            health: 600.0,
            shape: BlockShape::default(),
            cost: 150,
            components: vec![BlockComponent::Sprite(SpriteComponent {
                sprite: "textures/blocks/surge-wall-large.png".into(),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            })],
        },
    );

    registry.register(
        BlockType::Core,
        BlockDef {
            name: "Core".into(),
            description: "Core".into(),
            block_type: BlockType::Core,
            block_group: BlockGroup::Core,
            mass: 50.0,
            health: 600.0,
            shape: BlockShape::filled(3, 3),
            cost: 150,
            components: vec![
                BlockComponent::Sprite(SpriteComponent {
                    sprite: "textures/blocks/core-nucleus.png".into(),
                    offset: Vec2::ZERO,
                    z: 0.0,
                    animated: false,
                }),
                BlockComponent::Sprite(SpriteComponent {
                    sprite: "textures/blocks/core-nucleus-team.png".into(),
                    offset: Vec2::ZERO,
                    z: 0.1,
                    animated: false,
                }),
            ],
        },
    );

    registry.register(
        BlockType::LazerTurret,
        BlockDef {
            name: "Lazer Turret".into(),
            description: "Core".into(),
            block_type: BlockType::LazerTurret,
            block_group: BlockGroup::Turret,
            mass: 50.0,
            health: 600.0,
            shape: BlockShape::filled(1, 1),
            cost: 150,
            components: vec![
                BlockComponent::Sprite(SpriteComponent {
                    sprite: "textures/blocks/block-2.png".into(),
                    offset: Vec2::ZERO,
                    z: 0.0,
                    animated: false,
                }),
                BlockComponent::Turret(TurretComponent {
                    component_id: "lazer_turret".into(),
                    range: 20000.0,
                    turn_speed: 2.0,
                    barrels: vec![Vec2::new(0.0, 0.0)],
                    fire_all_at_once: false,
                    strategy: TargetingStrategy::Closest,
                    sprite: "textures/blocks/lancer.png".into(),
                    offset: Vec2::ZERO,
                    z: 0.1,
                    animated: false,
                    shooting_pattern: ShootPattern::Single {
                        fire_rate: 0.15,
                        projectile: ShootingProjectile::Bullet {
                            speed: 2000.0,
                            damage: 20.0,
                            lifetime: 5.0,
                            piercing: 0,
                        },
                    },
                }),
            ],
        },
    );
}
