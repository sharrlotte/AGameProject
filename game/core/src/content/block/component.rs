use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::{
    constant::GRID_SIZE,
    content::{
        block::feature::{component::Feature, turret::TurretComponent},
        ship::{grid::ShipGrid, Ship, ShipFragment},
        team::Team,
    },
};

#[derive(Clone, PartialEq, Reflect)]
pub struct RecoilComponent {
    pub strength: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct HeatEmitterComponent {
    pub amount: f32,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum ShootingProjectile {
    Bullet {
        speed: f32,
        damage: f32,
        lifetime: f32,
        piercing: i8,
        width: f32,
        length: f32,
    },
    Laser {
        damage: f32,
        width: f32,
    },
    Missile {
        speed: f32,
        damage: f32,
        lifetime: f32,
        lock_range: f32,
        blast_damage: f32,
        blast_radius: f32,
        width: f32,
        length: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct SingleShootPattern {
    pub fire_rate: f32,
    pub projectile: ShootingProjectile,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct SingleShootPatternState {
    pub last_fired_at: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct BurstShootPattern {
    pub fire_rate: f32,
    pub delay: f32,
    pub amount: i8,
    pub projectile: ShootingProjectile,
    pub shoot_sound: Option<Handle<AudioSource>>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum BurstShootPatternShootingState {
    Shooting {
        current_barrel: usize,
        last_barrel_fired_at: f32,
        current_amount: i8,
    },
    Cooldown {
        last_fired_at: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct BurstShootPatternState {
    pub shooting: BurstShootPatternShootingState,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct LaserBeamPattern {
    pub cooldown: f32,
    pub duration: f32,
    pub damage: f32,
    pub width: f32,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum LaserBeamPatternShootingState {
    Shooting { start_time: f32 },
    Cooldown { last_fired_at: f32 },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct PullingBeamPattern {
    pub width: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct PullingBeamPatternState {
    pub beam: Option<Entity>,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct LaserBeamPatternState {
    pub targets: Vec<Entity>,
    pub beam: Option<Entity>,
    pub shooting: LaserBeamPatternShootingState,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum ShootPattern {
    Single(SingleShootPattern),
    Burst(BurstShootPattern),
    LaserBeam(LaserBeamPattern),
    PullingBeam(PullingBeamPattern),
}

impl ShootPattern {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        match self {
            ShootPattern::Single(s) => {
                commands
                    .entity(base)
                    .insert((s.clone(), SingleShootPatternState { last_fired_at: 0.0 }));
            }
            ShootPattern::Burst(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    BurstShootPatternState {
                        shooting: BurstShootPatternShootingState::Cooldown { last_fired_at: 0.0 },
                    },
                ));
            }
            ShootPattern::LaserBeam(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    LaserBeamPatternState {
                        targets: Vec::new(),
                        beam: None,
                        shooting: LaserBeamPatternShootingState::Cooldown { last_fired_at: 0.0 },
                    },
                ));
            }
            ShootPattern::PullingBeam(s) => {
                commands
                    .entity(base)
                    .insert((s.clone(), PullingBeamPatternState { beam: None }));
            }
        }
    }
}

#[derive(Component, Clone, PartialEq)]
pub struct Block {
    pub id: Uuid,
    pub rotation: u8,
    pub position: IVec2,
    pub block_type: String,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockState {
    pub block_type: String,
}

#[derive(Component, Clone, PartialEq)]
pub struct BlockOwner {
    // Ship entity
    pub id: Entity,
}

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerProducer {
    pub output: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: String,
    pub rotation: u8,
}

impl BlockData {
    pub fn new(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            position: self.position,
            block_type: self.block_type.clone(),
            rotation: self.rotation,
        }
    }

    pub fn position_pixel(&self) -> Vec2 {
        self.position.as_vec2() * GRID_SIZE
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        def: &BlockDef,
        ship: Entity,
        team: Team,
        config: &GameConfigValue,
    ) -> Entity {
        let half_size = def.get_pixel_size() / 2.0;
        let mut transform = def.to_transform(self.position);

        // Use the first texture's z as the block sprite's render layer so the
        // collapsed sprite keeps its original visual stacking.
        if let Some(z) = def
            .features
            .iter()
            .find_map(|f| f.extra_texture().map(|t| t.z))
        {
            transform.translation.z = z;
        }

        let block = commands
            .spawn((
                transform,
                team,
                Visibility::Visible,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(half_size.x, half_size.y),
                CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
                ColliderMassProperties::Mass(0.0),
                Sensor,
                Block {
                    id: self.id,
                    block_type: self.block_type.clone(),
                    rotation: self.rotation,
                    position: self.position,
                },
                BlockState {
                    block_type: self.block_type.clone(),
                },
                Health {
                    health: def.health,
                    max_health: def.health,
                },
                BlockOwner { id: ship },
            ))
            .id();

        commands.entity(ship).add_child(block);

        let mut textures = def.features.iter().filter_map(|f| f.extra_texture());
        textures.next(); // primary texture is attached to the block entity
        for extra in textures {
            let layer = commands
                .spawn((
                    Transform::from_translation(extra.offset.extend(extra.z)),
                    Sprite {
                        image: extra.image.clone(),
                        custom_size: Some(def.get_pixel_size()),
                        ..default()
                    },
                ))
                .id();
            commands.entity(block).add_child(layer);
        }

        for feature in &def.features {
            feature.insert(commands, block, def, &team, config);
        }

        block
    }
}

#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block_type: String,
    pub block_id: Uuid,
    pub ship: Entity,
}

pub const FACE_RIGHT: u8 = 0b0001; // +X
pub const FACE_LEFT: u8 = 0b0010; // -X
pub const FACE_UP: u8 = 0b0100; // +Y
pub const FACE_DOWN: u8 = 0b1000; // -Y
pub const FACE_ALL: u8 = 0b1111;

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
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

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub struct BlockShape {
    pub size: IVec2,
    pub voxels: Vec<BlockVoxel>,
}

impl Default for BlockShape {
    fn default() -> Self {
        Self {
            size: IVec2::ONE,
            voxels: vec![BlockVoxel::default()],
        }
    }
}

#[derive(Clone, PartialEq, Reflect, Debug, Serialize, Deserialize)]
pub struct ItemDrop {
    pub item_type: String,
    pub amount: u32,
}

#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
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
    pub block_type: String,
    pub block_group: String,
    pub mass: f32,
    pub health: f32,
    // TODO: Better shape data type for complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    pub features: Vec<Feature>,
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
    pub(crate) map: HashMap<String, BlockDef>,
    pub(crate) by_name: HashMap<String, String>,
}

pub struct ShipCalculatedData {
    pub data: Ship,
    pub size: Vec2,
    pub blocks: Vec<BlockData>,
}

pub struct ShipCalculatedStat {
    pub ship: Option<ShipCalculatedData>,
    pub fragments: Vec<ShipFragment>,
}

impl BlockRegistry {
    pub fn register(&mut self, key: &str, def: BlockDef) {
        if self.map.contains_key(&def.block_type) {
            warn!("Block type {:#?} is already registered.", def.block_type);
            return;
        }

        self.by_name.insert(key.to_string(), def.block_type.clone());
        self.map.insert(def.block_type.clone(), def);
    }

    pub fn get(&self, key: &str) -> Option<&BlockDef> {
        let sym = self.by_name.get(key)?;
        self.map.get(sym)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &BlockDef)> {
        let mut value = self.map.iter().collect::<Vec<_>>();

        value.sort_by(|a, b| a.1.block_group.cmp(&b.1.block_group));

        return value.into_iter();
    }

    pub fn blocks_to_voxel_points(&self, grid: &ShipGrid) -> Vec<Vec2> {
        grid.occupied_cells()
            .map(|(cell, _)| cell.as_vec2() * GRID_SIZE)
            .collect()
    }

    pub fn build_ship_voxel_collider_2d(&self, grid: &ShipGrid) -> Collider {
        let points = self.blocks_to_voxel_points(grid);

        Collider::voxels_from_points(Vec2::new(GRID_SIZE, GRID_SIZE), &points)
    }
    pub fn blocks_stats(&self, blocks: &[BlockData], grid: &ShipGrid) -> ShipCalculatedStat {
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut ship: Option<ShipCalculatedData> = None;
        let mut fragments: Vec<ShipFragment> = Vec::new();

        for block in blocks {
            if visited.contains(&block.id) {
                continue;
            }

            let mut mass: f32 = 0.0;
            let mut core_position = None;

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

                for y in [-1, size.y] {
                    for x in 0..size.x {
                        let cell = IVec2::new(x, y) + block.position;
                        let Some(neighbor) = grid.block_at(cell) else {
                            continue;
                        };

                        queue.push(neighbor);
                    }
                }

                for x in [-1, size.x] {
                    for y in 0..size.y {
                        let cell = IVec2::new(x, y) + block.position;
                        let Some(neighbor) = grid.block_at(cell) else {
                            continue;
                        };

                        queue.push(neighbor);
                    }
                }

                max_x = max_x.max(block.position.x + size.x);
                max_y = max_y.max(block.position.y + size.y);

                min_x = min_x.min(block.position.x);
                min_y = min_y.min(block.position.y);

                mass += def.mass;

                if def.block_group == "core" {
                    let size = def.shape.size.as_vec2();
                    let offset = size * GRID_SIZE * 0.5;
                    core_position = Some(block.position.as_vec2() * GRID_SIZE + offset);
                }

                for feature in &def.features {
                    match feature {
                        Feature::Turret(TurretComponent { range, .. }) => {
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
                    data: Ship {
                        id: block.id,
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
                    });
                }
            }
        }

        return ShipCalculatedStat { ship, fragments };
    }
}

#[derive(Component, Debug)]
pub struct FlashOverlay {
    pub timer: Timer,
}
