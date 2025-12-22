use bevy::prelude::*;
use std::collections::HashMap;

use crate::content::block::base::{
    BlockComponent, BlockDef, BlockGroup, BlockShape, BlockType, BurstShootPattern, ShootPattern,
    ShootingProjectile, SingleShootPattern, SpriteComponent, TargetingStrategy, TurretComponent,
};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    map: HashMap<BlockType, BlockDef>,
}

impl BlockRegistry {
    fn register(&mut self, def: BlockDef) {
        if self.map.contains_key(&def.block_type) {
            warn!("Block type {:#?} is already registered.", def.block_type);
            return;
        }

        self.map.insert(def.block_type, def);
    }

    pub fn get(&self, block_type: &BlockType) -> Option<&BlockDef> {
        self.map.get(block_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&BlockType, &BlockDef)> {
        self.map.iter()
    }
}

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>, assets: Res<AssetServer>) {
    let load = |image: String| -> Handle<Image> { assets.load(image) };

    registry.register(BlockDef {
        name: "Standard Hull".into(),
        block_type: BlockType::Hull,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 100.0,
        shape: BlockShape::default(),
        cost: 5,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Small Ion Thruster".into(),
        block_type: BlockType::ThrusterSmall,
        block_group: BlockGroup::Base,
        mass: 25.0,
        health: 5000.0,
        shape: BlockShape::default(),
        cost: 50,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/thruster.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Micro Fusion Reactor".into(),
        block_type: BlockType::ReactorSmall,
        block_group: BlockGroup::Base,

        mass: 1.0,
        health: 800.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Command Cockpit".into(),
        block_type: BlockType::Cockpit,
        block_group: BlockGroup::Base,
        mass: 5.0,
        health: 600.0,
        shape: BlockShape::default(),
        cost: 150,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Core".into(),
        block_type: BlockType::Core,
        block_group: BlockGroup::Core,
        mass: 50.0,
        health: 600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: load("textures/blocks/core-nucleus.png".into()),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            }),
            BlockComponent::Sprite(SpriteComponent {
                image: load("textures/blocks/core-nucleus-team.png".into()),
                offset: Vec2::ZERO,
                z: 0.1,
                animated: false,
            }),
        ],
    });

    registry.register(BlockDef {
        name: "Lazer Turret".into(),
        block_type: BlockType::LazerTurret,
        block_group: BlockGroup::Turret,
        mass: 15.0,
        health: 600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: load("textures/blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 20000.0,
                turn_speed: 4.0,
                barrels: vec![Vec2::new(0.0, 0.0)],
                strategy: TargetingStrategy::Closest,
                image: load("textures/blocks/lancer.png".into()),
                offset: Vec2::ZERO,
                z: 0.1,
                animated: false,
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 3.0,
                    delay: 0.075,
                    amount: 9,
                    projectile: ShootingProjectile::Bullet {
                        speed: 5000.0,
                        damage: 20.0,
                        lifetime: 5.0,
                        piercing: 0,
                    },
                }),
            }),
        ],
    });

    registry.register(BlockDef {
        name: "Scrap Block".into(),
        block_type: BlockType::BlockScrap,
        block_group: BlockGroup::Base,
        mass: 1.0,
        health: 800.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/block-scrap.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Steel Block".into(),
        block_type: BlockType::BlockSteel,
        block_group: BlockGroup::Base,
        mass: 2.0,
        health: 1600.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: load("textures/blocks/block-steel.png".into()),
            offset: Vec2::ZERO,
            z: 0.0,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Swarmer Turret".into(),
        block_type: BlockType::Swarmer,
        block_group: BlockGroup::Turret,
        mass: 15.0,
        health: 600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: load("textures/blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 20000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: load("textures/blocks/swarmer.png".into()),
                offset: Vec2::ZERO,
                z: 0.1,
                animated: false,
                barrels: vec![Vec2::new(32.0, 0.0), Vec2::new(-32.0, 0.0)],
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 2.0,
                    projectile: ShootingProjectile::Missile {
                        speed: 3000.0,
                        damage: 20.0,
                        lifetime: 5.0,
                        lock_range: 6000.0,
                        blast_damage: 20.0,
                        blast_radius: 128.0,
                    },
                }),
            }),
        ],
    });
   
    registry.register(BlockDef {
        name: "Spectre Turret".into(),
        block_type: BlockType::Spectre,
        block_group: BlockGroup::Turret,
        mass: 15.0,
        health: 600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: load("textures/blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: 0.0,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 20000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: load("textures/blocks/spectre.png".into()),
                offset: Vec2::ZERO,
                z: 0.1,
                animated: false,
                barrels: vec![Vec2::new(32.0, 0.0), Vec2::new(-32.0, 0.0)],
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 0.2,
                    delay: 0.2,
                    amount: 12,
                    projectile: ShootingProjectile::Bullet {
                        speed: 4000.0,
                        damage: 20.0,
                        lifetime: 5.0,
                        piercing: 2,
                    },
                }),
            }),
        ],
    });
}
