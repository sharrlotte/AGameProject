use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;
use std::collections::HashMap;

use crate::{
    constant::{
        GRID_SIZE, LAYER_BLOCK, LAYER_TURRET_OVERLAY, THRUST_POWER_FACTOR, TURN_SPEED_FACTOR,
    },
    content::block::component::{
        BlockComponent, BlockData, BlockDef, BlockGroup, BlockShape, BlockType, BurstShootPattern,
        ShootPattern, ShootingProjectile, SingleShootPattern, SpriteComponent, TargetingStrategy,
        ThrusterComponent, TurretComponent,
    },
};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    map: HashMap<BlockType, BlockDef>,
}

pub struct ShipCalculatedStat {
    pub mass: f32,
    pub core_position: Vec2,
    pub center: Vec2,
    pub collider: Collider,
    pub thrust_power: f32,
    pub turn_speed: f32,
    pub size: Vec2,
    pub min_range: f32,
    pub max_range: f32,
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

    pub fn blocks_to_voxel_points(&self, blocks: &[BlockData]) -> Vec<Vec2> {
        let mut points = Vec::new();

        for block in blocks {
            let def = self.get(&block.block_type).unwrap();
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

    pub fn build_ship_voxel_collider_2d(&self, blocks: &[BlockData]) -> Collider {
        let points = self.blocks_to_voxel_points(blocks);

        Collider::voxels_from_points(Vec2::new(GRID_SIZE, GRID_SIZE), &points)
    }

    pub fn blocks_stats(&self, blocks: &[BlockData]) -> ShipCalculatedStat {
        let mut mass: f32 = 0.0;
        let mut core_position = Vec2::ZERO;
        let mut thrust_power: f32 = 1000.0;
        let mut turn_speed: f32 = 250.0;

        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        let mut min_x: i32 = 0;
        let mut min_y: i32 = 0;

        let mut min_range: f32 = f32::MAX;
        let mut max_range: f32 = f32::MIN;

        for block in blocks {
            let Some(def) = self.get(&block.block_type) else {
                warn!("Block type {:?} not found in registry", block.block_type);
                continue;
            };

            let size = def.shape.size;

            max_x = max_x.max(block.position.x + size.x);
            max_y = max_y.max(block.position.y + size.y);

            min_x = min_x.min(block.position.x);
            min_y = min_y.min(block.position.y);

            mass += def.mass;

            if def.block_group == BlockGroup::Core {
                let size = def.shape.size.as_vec2();
                let offset = size * GRID_SIZE * 0.5;
                core_position = block.position.as_vec2() * GRID_SIZE + offset;
            }

            for component in &def.components {
                match component {
                    BlockComponent::Thruster(ThrusterComponent { force, torque }) => {
                        thrust_power += force;
                        turn_speed += torque;
                    }
                    BlockComponent::Turret(TurretComponent { range, .. }) => {
                        min_range = min_range.min(*range);
                        max_range = max_range.max(*range);
                    }
                    _ => {}
                }
            }
        }

        return ShipCalculatedStat {
            thrust_power: thrust_power * THRUST_POWER_FACTOR,
            turn_speed: turn_speed * TURN_SPEED_FACTOR,
            mass,
            core_position: core_position,
            min_range,
            max_range,
            collider: self.build_ship_voxel_collider_2d(&blocks),
            size: Vec2::new(
                (max_x - min_x) as f32 * GRID_SIZE,
                (max_y - min_y) as f32 * GRID_SIZE,
            ),
            center: Vec2::new(
                (max_x + min_x) as f32 * 0.5 * GRID_SIZE,
                (max_y + min_y) as f32 * 0.5 * GRID_SIZE,
            ),
        };
    }
}

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>, assets: Res<AssetServer>) {
    let image = |image: String| -> Handle<Image> { assets.load(format!("textures/{image}")) };
    let sound = |sound: String| -> Handle<AudioSource> { assets.load(format!("audio/{sound}")) };

    registry.register(BlockDef {
        name: "Standard Hull".into(),
        block_type: BlockType::Hull,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 1000.0,
        shape: BlockShape::default(),
        cost: 5,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
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
            image: image("blocks/thruster.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
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
            image: image("blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Command Cockpit".into(),
        block_type: BlockType::Cockpit,
        block_group: BlockGroup::Base,
        mass: 5.0,
        health: 1600.0,
        shape: BlockShape::default(),
        cost: 150,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("blocks/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Core".into(),
        block_type: BlockType::Core,
        block_group: BlockGroup::Core,
        mass: 50.0,
        health: 1600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("blocks/core-nucleus.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Sprite(SpriteComponent {
                image: image("blocks/core-nucleus-team.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
            }),
        ],
    });

    registry.register(BlockDef {
        name: "Lazer Turret".into(),
        block_type: BlockType::LazerTurret,
        block_group: BlockGroup::Turret,
        mass: 15.0,
        health: 1600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 12000.0,
                turn_speed: 4.0,
                barrels: vec![Vec2::new(0.0, 0.0)],
                strategy: TargetingStrategy::Closest,
                image: image("blocks/lancer.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 3.0,
                    delay: 0.075,
                    amount: 6,
                    projectile: ShootingProjectile::Bullet {
                        speed: 5000.0,
                        damage: 20.0,
                        lifetime: 5.0,
                        piercing: 0,
                    },
                    shoot_sound: None,
                }),
            }),
        ],
    });

    registry.register(BlockDef {
        name: "Scrap Block".into(),
        block_type: BlockType::BlockScrap,
        block_group: BlockGroup::Base,
        mass: 1.0,
        health: 1800.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("blocks/block-scrap.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
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
            image: image("blocks/block-steel.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
    });

    registry.register(BlockDef {
        name: "Swarmer Turret".into(),
        block_type: BlockType::Swarmer,
        block_group: BlockGroup::Turret,
        mass: 15.0,
        health: 1800.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 6000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("blocks/swarmer.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(32.0, 0.0), Vec2::new(-32.0, 0.0)],
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 2.0,
                    projectile: ShootingProjectile::Missile {
                        speed: 3000.0,
                        damage: 10.0,
                        lifetime: 10.0,
                        lock_range: 6000.0,
                        blast_damage: 10.0,
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
        health: 3600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("blocks/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 10000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("blocks/spectre.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(32.0, 64.0), Vec2::new(-32.0, 64.0)],
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
                    shoot_sound: Some(sound("bullet/shoot-spectre.ogg".into())),
                }),
            }),
        ],
    });
}
