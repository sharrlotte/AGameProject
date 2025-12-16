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
