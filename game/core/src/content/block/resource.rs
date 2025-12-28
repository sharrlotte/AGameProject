use bevy::{platform::collections::HashSet, prelude::*};
use bevy_rapier2d::prelude::Collider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    constant::{
        GRID_SIZE, LAYER_BLOCK, LAYER_TURRET_OVERLAY, THRUST_POWER_FACTOR, TURN_SPEED_FACTOR,
    },
    content::{
        block::component::{
            BlockComponent, BlockData, BurstShootPattern, LaserBeamPattern, PullingBeamPattern,
            ShootPattern, ShootingProjectile, SingleShootPattern, SpriteComponent,
            TargetingStrategy, ThrusterComponent, TurretComponent,
        },
        item::resource::ItemType,
        ship::component::{Ship, ShipFragment},
    },
};

pub const FACE_RIGHT: u8 = 0b0001; // +X
pub const FACE_LEFT: u8 = 0b0010; // -X
pub const FACE_UP: u8 = 0b0100; // +Y
pub const FACE_DOWN: u8 = 0b1000; // -Y
pub const FACE_ALL: u8 = 0b1111;

#[derive(Component, Clone, PartialEq, Debug, Serialize, Deserialize, Reflect, Eq, Hash, Copy)]
pub enum BlockType {
    Hull,
    Frame,
    ThrusterSmall,
    ThrusterLarge,
    ReactorSmall,
    WeaponLaser,
    Cockpit,
    Core,
    LazerTurret,
    Swarmer,
    Spectre,
    Meldown,
    Foreshadow,
    Colaris,
    Paralax,

    // Real
    BlockScrap,
    BlockSteel,
}

#[derive(Component, Clone, PartialEq, Reflect, Copy, PartialOrd, Ord, Eq)]
pub enum BlockGroup {
    Core,
    Base,
    Turret,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockVoxel {
    pub position: IVec2,
    pub connectable_faces: u8,
}

impl Default for BlockVoxel {
    fn default() -> Self {
        Self {
            position: IVec2::ZERO,
            connectable_faces: FACE_ALL,
        }
    }
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockShape {
    pub size: IVec2,
    pub voxels: Vec<BlockVoxel>,
}

impl BlockShape {
    pub fn filled(width: i32, height: i32) -> Self {
        let mut voxels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                voxels.push(BlockVoxel {
                    position: IVec2::new(x, y),
                    connectable_faces: FACE_ALL,
                });
            }
        }
        Self {
            size: IVec2::new(width, height),
            voxels,
        }
    }
}

impl Default for BlockShape {
    fn default() -> Self {
        Self {
            size: IVec2::ONE,
            voxels: vec![BlockVoxel::default()],
        }
    }
}

#[derive(Clone, PartialEq, Reflect, Debug)]
pub struct ItemDrop {
    pub item_type: ItemType,
    pub amount: u32,
}

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum DropComponent {
    Item(ItemDrop),
}

#[derive(Message, Clone, PartialEq, Reflect)]
pub struct DropEvent {
    pub drop: DropComponent,
    pub position: Vec2,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockDef {
    pub name: String,
    pub block_type: BlockType,
    pub block_group: BlockGroup,
    pub mass: f32,
    pub health: f32,
    // Better shape data type for complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    pub components: Vec<BlockComponent>,
    pub drops: Vec<DropComponent>,
}

impl BlockDef {
    pub fn get_block_size(&self) -> Vec2 {
        return self.shape.size.as_vec2();
    }

    pub fn get_pixel_size(&self) -> Vec2 {
        return self.get_block_size() * GRID_SIZE;
    }

    pub fn to_transform(&self, position: IVec2) -> Transform {
        let anchor_position = Vec3::new(
            position.x as f32 * GRID_SIZE,
            position.y as f32 * GRID_SIZE,
            0.0,
        );

        let position = anchor_position + (self.get_pixel_size() / 2.0).extend(0.0);

        return Transform::from_translation(position);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    map: HashMap<BlockType, BlockDef>,
}

pub struct ShipCalculatedData {
    pub data: Ship,
    pub size: Vec2,
    pub center: Vec2,
    pub blocks: Vec<BlockData>,
}

pub struct ShipCalculatedStat {
    pub ship: Option<ShipCalculatedData>,
    pub fragments: Vec<ShipFragment>,
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
        let mut value = self.map.iter().collect::<Vec<_>>();

        value.sort_by(|a, b| a.1.block_group.cmp(&b.1.block_group));

        return value.into_iter();
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
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut cell_map: HashMap<IVec2, &BlockData> = HashMap::new();
        let mut ship: Option<ShipCalculatedData> = None;
        let mut fragments: Vec<ShipFragment> = Vec::new();

        for block in blocks {
            let Some(def) = self.get(&block.block_type) else {
                warn!("Block type {:?} not found in registry", block.block_type);
                continue;
            };

            for y in 0..def.shape.size.y {
                for x in 0..def.shape.size.x {
                    let cell = IVec2::new(x, y) + block.position;
                    cell_map.entry(cell).insert_entry(block);
                }
            }
        }

        for block in blocks {
            if visited.contains(&block.id) {
                continue;
            }

            let mut mass: f32 = 0.0;
            let mut core_position = None;
            let mut thrust_power: f32 = 0.0;
            let mut turn_speed: f32 = 0.0;

            let mut max_x: i32 = 0;
            let mut max_y: i32 = 0;
            let mut min_x: i32 = 0;
            let mut min_y: i32 = 0;

            let mut min_range: f32 = 10000.0;
            let mut max_range: f32 = 1000.0;

            let mut queue: Vec<&BlockData> = Vec::new();
            let mut current_blocks: Vec<BlockData> = Vec::new();

            queue.push(block);

            while let Some(block) = queue.pop() {
                if visited.contains(&block.id) {
                    continue;
                }

                current_blocks.push(block.clone());
                visited.insert(block.id);

                let Some(def) = self.get(&block.block_type) else {
                    warn!("Block type {:?} not found in registry", block.block_type);
                    continue;
                };

                let size = def.shape.size;

                for y in [-1, size.y + 1] {
                    for x in 0..size.x {
                        let cell = IVec2::new(x, y) + block.position;
                        let Some(neighbor) = cell_map.get(&cell) else {
                            continue;
                        };

                        queue.push(*neighbor);
                    }
                }

                for x in [-1, size.x + 1] {
                    for y in 0..size.y {
                        let cell = IVec2::new(x, y) + block.position;
                        let Some(neighbor) = cell_map.get(&cell) else {
                            continue;
                        };

                        queue.push(*neighbor);
                    }
                }

                max_x = max_x.max(block.position.x + size.x);
                max_y = max_y.max(block.position.y + size.y);

                min_x = min_x.min(block.position.x);
                min_y = min_y.min(block.position.y);

                mass += def.mass;

                if def.block_group == BlockGroup::Core {
                    let size = def.shape.size.as_vec2();
                    let offset = size * GRID_SIZE * 0.5;
                    core_position = Some(block.position.as_vec2() * GRID_SIZE + offset);
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

            if let Some(core_position) = core_position {
                ship = Some(ShipCalculatedData {
                    size: Vec2::new(
                        (max_x - min_x) as f32 * GRID_SIZE,
                        (max_y - min_y) as f32 * GRID_SIZE,
                    ),
                    center: Vec2::new(
                        (max_x + min_x) as f32 * 0.5 * GRID_SIZE,
                        (max_y + min_y) as f32 * 0.5 * GRID_SIZE,
                    ),
                    data: Ship {
                        id: block.id,
                        thrust_power: thrust_power * THRUST_POWER_FACTOR,
                        turn_speed: turn_speed * TURN_SPEED_FACTOR,
                        core_position,
                        max_range,
                        min_range,
                        mass,
                    },
                    blocks: current_blocks,
                });
            } else {
                if current_blocks.len() > 0 {
                    fragments.push(ShipFragment {
                        blocks: current_blocks,
                        mass,
                    });
                }
            }
        }

        return ShipCalculatedStat { ship, fragments };
    }
}

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>, assets: Res<AssetServer>) {
    let image = |image: String| -> Handle<Image> { assets.load(format!("texture/{image}")) };
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
            image: image("block/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Small Ion Thruster".into(),
        block_type: BlockType::ThrusterSmall,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 5000.0,
        shape: BlockShape::default(),
        cost: 50,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/thruster.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Thruster(ThrusterComponent {
                force: 200.0,
                torque: 100.0,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Core".into(),
        block_type: BlockType::Core,
        block_group: BlockGroup::Core,
        mass: 50.0,
        health: 5000.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/core-nucleus.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/core-nucleus-team.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
            }),
            BlockComponent::Thruster(ThrusterComponent {
                force: 100.0,
                torque: 40.0,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Lazer Turret".into(),
        block_type: BlockType::LazerTurret,
        block_group: BlockGroup::Turret,
        mass: 7.0,
        health: 1600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 5000.0,
                turn_speed: 4.0,
                barrels: vec![Vec2::new(0.0, 0.0)],
                strategy: TargetingStrategy::Closest,
                image: image("block/lancer.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 3.0,
                    delay: 0.25,
                    amount: 6,
                    projectile: ShootingProjectile::Laser {
                        damage: 40.0,
                        width: 3.0,
                    },
                    shoot_sound: None,
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Scrap Block".into(),
        block_type: BlockType::BlockScrap,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 1800.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("block/block-iron.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Steel Block".into(),
        block_type: BlockType::BlockSteel,
        block_group: BlockGroup::Base,
        mass: 8.0,
        health: 1600.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("block/block-steel.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Swarmer Turret".into(),
        block_type: BlockType::Swarmer,
        block_group: BlockGroup::Turret,
        mass: 8.0,
        health: 1800.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 3000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/swarmer.png".into()),
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
                        lock_range: 3000.0,
                        blast_damage: 30.0,
                        blast_radius: 32.0,
                        width: 5.0,
                        length: 60.0,
                    },
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Spectre Turret".into(),
        block_type: BlockType::Spectre,
        block_group: BlockGroup::Turret,
        mass: 16.0,
        health: 3600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 5000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/spectre.png".into()),
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
                        lifetime: 10.0,
                        piercing: 2,
                        width: 3.0,
                        length: 60.0,
                    },
                    shoot_sound: Some(sound("bullet/shoot-spectre.ogg".into())),
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Meltdown Turret".into(),
        block_type: BlockType::Meldown,
        block_group: BlockGroup::Turret,
        mass: 20.0,
        health: 3600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 3000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/meltdown.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::LaserBeam(LaserBeamPattern {
                    cooldown: 10.0,
                    duration: 5.0,
                    damage: 1.0,
                    width: 9.0,
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Foreshadow Turret".into(),
        block_type: BlockType::Foreshadow,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(3, 9),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 15000.0,
                turn_speed: 0.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/foreshadow.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 10.0,
                    projectile: ShootingProjectile::Bullet {
                        speed: 20000.0,
                        damage: 5000.0,
                        lifetime: 30.0,
                        piercing: 10,
                        width: 12.0,
                        length: 1000.0,
                    },
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Colaris Turret".into(),
        block_type: BlockType::Colaris,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 6000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/colaris.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 2.0,
                    delay: 0.2,
                    amount: 12,
                    shoot_sound: None,
                    projectile: ShootingProjectile::Bullet {
                        speed: 4000.0,
                        damage: 30.0,
                        lifetime: 10.0,
                        piercing: 0,
                        width: 3.0,
                        length: 12.0,
                    },
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Paralax Turret".into(),
        block_type: BlockType::Paralax,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 6000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/colaris.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::PullingBeam(PullingBeamPattern { width: 3.0 }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });
}
