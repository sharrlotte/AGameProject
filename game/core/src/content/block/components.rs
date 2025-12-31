use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    constant::GRID_SIZE,
    content::{
        block::component::{components::BlockComponent, turret::TurretComponent},
        entity::EntityType,
        item::ItemType,
        selectable::Selectable,
        ship::{Ship, ShipFragment},
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

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
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

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
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

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
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

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct PullingBeamPattern {
    pub width: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
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
    pub block_type: BlockType,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockState {
    pub block_type: BlockType,
}

#[derive(Component, Clone, PartialEq)]
pub struct BlockOwner {
    // Ship entity
    pub id: Entity,
}

#[derive(Component, Clone, PartialEq)]
pub struct ComponentOwner {
    // Block entity
    pub id: Entity,
}

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerConsumer {
    pub required_input: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerProducer {
    pub output: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerStorage {
    pub capacity: f32,
    pub current: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

impl BlockData {
    pub fn new(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            position: self.position,
            block_type: self.block_type,
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
    ) -> Entity {
        let half_size = def.get_pixel_size() / 2.0;
        let transform = def.to_transform(self.position);

        let child = commands
            .spawn((
                transform,
                def.block_group,
                team,
                Visibility::Visible,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(half_size.x, half_size.y),
                CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
                ColliderMassProperties::Mass(0.0),
                Sensor,
                Block {
                    id: self.id,
                    block_type: self.block_type,
                    rotation: self.rotation,
                    position: self.position,
                },
                BlockState {
                    block_type: self.block_type,
                },
                Health {
                    health: def.health,
                    max_health: def.health,
                },
                BlockOwner { id: ship },
                EntityType::Block,
                Selectable,
            ))
            .id();

        commands.entity(ship).add_child(child);

        for component in &def.components {
            component.spawn(commands, child, def, &team);
        }

        return child;
    }
}

#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block_type: BlockType,
    pub block_id: Uuid,
    pub ship: Entity,
}

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
    // TODO: Better shape data type for complex voxel structures
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
    pub(crate) map: HashMap<BlockType, BlockDef>,
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
    pub fn register(&mut self, def: BlockDef) {
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
                        mass,
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
