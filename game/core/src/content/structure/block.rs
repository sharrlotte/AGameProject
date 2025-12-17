use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the type of a block.
/// This acts as a key to look up static properties (Mass, Health, etc.) in the BlockRegistry.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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
}

/// Component attached to every block entity.
#[derive(Component, Debug, Clone, Reflect)]
pub struct Block {
    pub block_type: BlockType,
    /// The current rotation of the block in 90-degree increments (0=0deg, 1=90deg, 2=180deg, 3=270deg)
    pub rotation: u8,
}

/// Represents a single voxel within a block's local grid.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Voxel {
    /// Local coordinate relative to block origin.
    pub pos: IVec2,
    /// Bitmask of faces that can connect to other blocks (Up, Down, Left, Right).
    pub connectable_faces: u8,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            pos: IVec2::ZERO,
            connectable_faces: FACE_ALL,
        }
    }
}

/// Defines the physical shape and connectivity of a block in 2D.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockShape {
    /// Bounding box dimensions (width, height)
    pub size: IVec2,
    /// List of occupied local coordinates with their connectivity info.
    pub voxels: Vec<Voxel>,
}

impl BlockShape {
    pub fn filled(width: i32, height: i32) -> Self {
        let mut voxels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                voxels.push(Voxel {
                    pos: IVec2::new(x, y),
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
            voxels: vec![Voxel::default()],
        }
    }
}

/// Defines the position of a block within the ship's 2D grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct GridPosition(pub IVec2);

// Bitmasks for faces in 2D
pub const FACE_RIGHT: u8 = 0b0001; // +X
pub const FACE_LEFT: u8 = 0b0010; // -X
pub const FACE_UP: u8 = 0b0100; // +Y
pub const FACE_DOWN: u8 = 0b1000; // -Y
pub const FACE_ALL: u8 = 0b1111;

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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum LayerKind {
    Base,
    Turret,
    Engine,
    Sensor,
    Decorative,
    // keep this SMALL
}

impl Default for LayerKind {
    fn default() -> Self {
        Self::Base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct LayerVisual {
    pub sprite: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]

pub enum LayerFeature {
    Rotatable(RotatableFeature),
    Shooter(ShooterFeature),
    Tracker(TrackerFeature),
    Recoil(RecoilFeature),
    HeatEmitter(HeatEmitterFeature),
    EnergyConsumer(EnergyConsumerFeature),
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RotatableFeature {
    pub turn_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ShooterFeature {
    pub fire_rate: f32,
    pub projectile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RecoilFeature {
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TrackerFeature {
    pub range: f32,
    pub fov: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct HeatEmitterFeature {
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct EnergyConsumerFeature {
    pub rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockLayerDefinition {
    pub id: String,

    /// e.g. "base", "turret", "engine", "radar"
    pub kind: LayerKind,

    /// Visual parts for this layer
    pub visuals: Vec<LayerVisual>,

    /// Optional feature payloads (composable)
    pub features: Vec<LayerFeature>,
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
    pub layers: Vec<BlockLayerDefinition>,
}
