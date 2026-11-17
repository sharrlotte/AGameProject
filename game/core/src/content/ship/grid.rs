use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::content::block::{BlockData, BlockDef, BlockRegistry};

/// Per-ship cell grid. `blocks` maps cell → block entity (editor placement);
/// `cells` maps cell → block data (runtime cache for stats/collider rebuilds).
#[derive(Component, Debug, Default, Reflect)]
pub struct ShipGrid {
    #[reflect(ignore)]
    pub blocks: HashMap<IVec2, Entity>,
    #[reflect(ignore)]
    pub cells: HashMap<IVec2, BlockData>,
}

impl ShipGrid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, position: IVec2, entity: Entity) {
        self.blocks.insert(position, entity);
    }

    pub fn get(&self, position: IVec2) -> Option<&Entity> {
        self.blocks.get(&position)
    }

    pub fn remove(&mut self, position: IVec2) -> Option<Entity> {
        self.blocks.remove(&position)
    }

    pub fn block_at(&self, cell: IVec2) -> Option<&BlockData> {
        self.cells.get(&cell)
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = (&IVec2, &BlockData)> {
        self.cells.iter()
    }

    /// Insert one block's occupied cells into the cache.
    pub fn insert_block(&mut self, block: &BlockData, def: &BlockDef) {
        for y in 0..def.shape.size.y {
            for x in 0..def.shape.size.x {
                self.cells
                    .insert(IVec2::new(x, y) + block.position, block.clone());
            }
        }
    }

    /// Remove every cell owned by a block id.
    pub fn remove_block(&mut self, block_id: Uuid) {
        self.cells.retain(|_, data| data.id != block_id);
    }

    /// Full rebuild from a block list — the correctness fallback after editor
    /// spawns or ship loads. Not used on the normal destroy hot path.
    pub fn rebuild(&mut self, blocks: &[BlockData], registry: &BlockRegistry) {
        self.cells.clear();
        for block in blocks {
            if let Some(def) = registry.get(&block.block_type) {
                self.insert_block(block, def);
            }
        }
    }
}
