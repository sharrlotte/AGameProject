use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    // Real
    BlockScrap,
    BlockSteel,
}

#[derive(Component, Clone, PartialEq, Reflect, Copy)]
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

#[derive(Component, Clone, PartialEq)]
pub struct GridPosition(pub IVec2);

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum BlockComponent {
    Sprite(SpriteComponent),
    Turret(TurretComponent),
    Recoil(RecoilComponent),
    HeatEmitter(HeatEmitterComponent),
    EnergyConsumer(EnergyConsumerComponent),
    Thruster(ThrusterComponent),
}

#[derive(Clone, PartialEq, Reflect)]
pub struct RecoilComponent {
    pub strength: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct HeatEmitterComponent {
    pub amount: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct EnergyConsumerComponent {
    pub rate: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct SpriteComponent {
    pub sprite: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct TurretComponent {
    pub component_id: String,
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub fire_all_at_once: bool,
    pub shooting_pattern: ShootPattern,
    pub sprite: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
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
    },
    Beam {
        range: f32,
        damage_per_second: f32,
        width: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub enum ShootPattern {
    Single {
        fire_rate: f32,
        projectile: ShootingProjectile,
    },
    Burst {
        count: u32,
        delay: f32,
        projectile: ShootingProjectile,
    },
    Continuous,
}

impl ShootPattern {
    pub fn new(&self) -> ShootPatternState {
        match self {
            ShootPattern::Single { .. } => ShootPatternState::Single { last_fired_at: 0.0 },
            ShootPattern::Burst { .. } => ShootPatternState::Burst {
                current_shot: 0,
                burst_timer: 0.0,
                is_bursting: false,
            },
            ShootPattern::Continuous => ShootPatternState::Continuous,
        }
    }
}

#[derive(Component, Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum ShootPatternState {
    Single {
        last_fired_at: f32,
    },
    Burst {
        current_shot: u32,
        burst_timer: f32,
        is_bursting: bool,
    },
    Continuous,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockDef {
    pub name: String,
    pub description: String,
    pub block_type: BlockType,
    pub block_group: BlockGroup,
    pub mass: f32,
    pub health: f32,
    // Better shape data type for complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    pub components: Vec<BlockComponent>,
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
pub struct ShipIdComponent {
    pub ship: Entity,
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

#[derive(Component, Debug, Clone, Reflect)]
pub struct Thruster {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct ThrusterComponent {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Weapon {
    pub damage: f32,
    pub cooldown: f32,
    pub range: f32,
}
