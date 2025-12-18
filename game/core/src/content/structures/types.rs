use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::turret::types::TurretFeature;

/// Represents the type of a block.
/// This acts as a key to look up static properties (Mass, Health, etc.) in the BlockRegistry.
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
}

#[derive(Component, Clone, PartialEq, Reflect, Copy)]
pub enum BlockGroup {
    Core,
    Base,
    Turret,
}

/// Represents a single voxel within a block's local grid.
#[derive(Clone, PartialEq, Reflect)]
pub struct BlockVoxel {
    /// Local coordinate relative to block origin.
    pub pos: IVec2,
    /// Bitmask of faces that can connect to other blocks (Up, Down, Left, Right).
    pub connectable_faces: u8,
}

// Bitmasks for faces in 2D
pub const FACE_RIGHT: u8 = 0b0001; // +X
pub const FACE_LEFT: u8 = 0b0010; // -X
pub const FACE_UP: u8 = 0b0100; // +Y
pub const FACE_DOWN: u8 = 0b1000; // -Y
pub const FACE_ALL: u8 = 0b1111;

impl Default for BlockVoxel {
    fn default() -> Self {
        Self {
            pos: IVec2::ZERO,
            connectable_faces: FACE_ALL,
        }
    }
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockShape {
    /// Bounding box dimensions (width, height)
    pub size: IVec2,
    /// List of occupied local coordinates with their connectivity info.
    pub voxels: Vec<BlockVoxel>,
}

impl BlockShape {
    pub fn filled(width: i32, height: i32) -> Self {
        let mut voxels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                voxels.push(BlockVoxel {
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
            voxels: vec![BlockVoxel::default()],
        }
    }
}

/// Defines the position of a block within the ship's 2D grid.
#[derive(Component, Clone, PartialEq)]
pub struct GridPosition(pub IVec2);

/// Enum representing distinct features/behaviors a block can have.
/// This allows us to extend block functionality without growing the base struct.
#[derive(Clone, PartialEq, Reflect)]
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

#[derive(Clone, PartialEq, Reflect)]
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

#[derive(Clone, PartialEq, Reflect)]
pub struct RecoilFeature {
    pub strength: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct HeatEmitterFeature {
    pub amount: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct EnergyConsumerFeature {
    pub rate: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct SpriteFeature {
    pub sprite: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

#[derive(Clone, PartialEq, Reflect)]
pub enum LayerFeature {
    Sprite(SpriteFeature),
    Turret(TurretFeature),
    Recoil(RecoilFeature),
    HeatEmitter(HeatEmitterFeature),
    EnergyConsumer(EnergyConsumerFeature),
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockLayerDef {
    pub id: String,
    /// e.g. "base", "turret", "engine", "radar"
    pub kind: LayerKind,
    /// Optional feature payloads (composable)
    pub features: Vec<LayerFeature>,
}

/// A definition struct that holds the static data for a block type.
/// This allows us to define blocks once and look them up later.
#[derive(Clone, PartialEq, Reflect)]
pub struct BlockDef {
    pub name: String,
    pub description: String,
    pub block_type: BlockType,
    pub block_group: BlockGroup,
    pub mass: f32,
    pub health: f32,
    /// Replaced `size` with `shape` to support complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    /// A list of features this block possesses.
    pub features: Vec<BlockFeature>,
    pub layers: Vec<BlockLayerDef>,
}

/// Component, Clone,  PartialEq attached to every block entity.
#[derive(Component, Clone, PartialEq)]
pub struct Block {
    pub id: usize,
    pub rotation: u8,
    pub block_type: BlockType,
}

#[derive(Component, Clone, PartialEq)]
pub struct PlacedBlock;

#[derive(Component, Clone, PartialEq)]
pub struct BlockLayer {
    pub kind: LayerKind,
}

#[derive(Component, Clone, PartialEq)]
pub struct BelongsToShip {
    pub ship: Entity,
}
