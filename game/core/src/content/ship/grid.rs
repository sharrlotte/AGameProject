use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::content::block::{BlockDef, BlockPlacement, BlockRegistry, GridBlock};

/// Per-ship block grid. This is authoritative for placement and identity:
/// `blocks` owns every live block by id, and `cells` maps occupied cells back
/// to a block id. Block entities only carry `BlockRef`/`BlockOwner` plus
/// runtime state.
#[derive(Component, Debug, Default)]
pub struct ShipGrid {
    pub blocks: HashMap<Uuid, GridBlock>,
    pub cells: HashMap<IVec2, Uuid>,
}

impl ShipGrid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert one block and fill every cell its shape occupies.
    pub fn insert_block(&mut self, placement: BlockPlacement, entity: Entity, def: &BlockDef) {
        for y in 0..def.shape.size.y {
            for x in 0..def.shape.size.x {
                self.cells
                    .insert(placement.position + IVec2::new(x, y), placement.id);
            }
        }
        self.blocks
            .insert(placement.id, GridBlock { placement, entity });
    }

    pub fn set_entity(&mut self, id: Uuid, entity: Entity) {
        if let Some(block) = self.blocks.get_mut(&id) {
            block.entity = entity;
        }
    }

    pub fn entity_at(&self, cell: IVec2) -> Option<Entity> {
        let id = self.cells.get(&cell)?;
        self.blocks.get(id).map(|block| block.entity)
    }

    pub fn block(&self, id: Uuid) -> Option<&GridBlock> {
        self.blocks.get(&id)
    }

    pub fn block_at(&self, cell: IVec2) -> Option<&GridBlock> {
        let id = self.cells.get(&cell)?;
        self.blocks.get(id)
    }

    /// Remove a block and all of its occupied cells.
    pub fn remove_block(&mut self, id: Uuid) -> Option<GridBlock> {
        self.cells.retain(|_, cell_id| *cell_id != id);
        self.blocks.remove(&id)
    }

    pub fn blocks(&self) -> impl Iterator<Item = &GridBlock> {
        self.blocks.values()
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = (&IVec2, &Uuid)> {
        self.cells.iter()
    }

    /// Rebuild the grid from a static layout. Entities start as placeholders;
    /// spawn code calls `set_entity` after creating each block entity.
    pub fn rebuild_from_layout(&mut self, layout: &[BlockPlacement], registry: &BlockRegistry) {
        self.blocks.clear();
        self.cells.clear();

        for placement in layout {
            let Some(def) = registry.get_symbol(&placement.block_type) else {
                warn!("Unknown block type {:?} in layout", placement.block_type);
                continue;
            };
            self.insert_block(placement.clone(), Entity::PLACEHOLDER, def);
        }
    }

    /// Recompute the cell map after block entities changed.
    pub fn refresh_cells(&mut self, registry: &BlockRegistry) {
        self.cells.clear();
        let blocks: Vec<(Uuid, IVec2, IVec2)> = self
            .blocks
            .iter()
            .filter_map(|(id, block)| {
                let def = registry.get_symbol(&block.placement.block_type)?;
                Some((*id, block.placement.position, def.shape.size))
            })
            .collect();

        for (id, position, size) in blocks {
            for y in 0..size.y {
                for x in 0..size.x {
                    self.cells.insert(position + IVec2::new(x, y), id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::GRID_SIZE;
    use crate::content::block::{BlockDef, BlockRegistry, BlockShape};

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BlockShape::default(),
            cost: 1,
            features: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn registry() -> BlockRegistry {
        let mut registry = BlockRegistry::default();
        registry.register("core", def("core", "core", 10.0, 100.0));
        registry.register("plate", def("plate", "structure", 5.0, 50.0));
        registry
    }

    fn layout() -> Vec<BlockPlacement> {
        vec![
            BlockPlacement::new(IVec2::ZERO, "core", 0),
            BlockPlacement::new(IVec2::X, "plate", 0),
        ]
    }

    #[test]
    fn cells_follow_shapes_and_removals() {
        let registry = registry();
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(&layout(), &registry);

        assert_eq!(grid.occupied_cells().count(), 2);
        assert!(grid.entity_at(IVec2::X).is_some());

        let plate_id = grid.block_at(IVec2::X).unwrap().placement.id;
        grid.remove_block(plate_id);
        assert_eq!(grid.occupied_cells().count(), 1);
        assert!(grid.block_at(IVec2::X).is_none());
    }

    #[test]
    fn stats_and_bounds_come_from_grid() {
        let registry = registry();
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(&layout(), &registry);

        let stats = registry.stats_from_grid(&grid).unwrap();
        assert_eq!(stats.mass, 15.0);
        assert_eq!(stats.max_health, 150.0);
        assert_eq!(stats.health, 150.0);

        let bounds = registry.bounds_from_grid(&grid).unwrap();
        assert_eq!(bounds.size, Vec2::new(GRID_SIZE * 2.0, GRID_SIZE));

        let (core_id, _) = registry.core_of(&grid).unwrap();
        assert_eq!(core_id, grid.block_at(IVec2::ZERO).unwrap().placement.id);
    }

    #[test]
    fn core_loss_becomes_fragment() {
        let registry = registry();
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(&layout(), &registry);

        let core_id = grid.block_at(IVec2::ZERO).unwrap().placement.id;
        grid.remove_block(core_id);

        assert!(registry.core_of(&grid).is_none());
        assert!(registry.stats_from_grid(&grid).is_none());

        let build = registry.build_ship_result(&grid);
        assert!(build.ship.is_none());
        assert_eq!(build.fragments.len(), 1);
        assert_eq!(build.fragments[0].placements.len(), 1);
    }

    #[test]
    fn validate_rejects_overlap_and_missing_core() {
        let registry = registry();

        let mut overlapping = layout();
        overlapping.push(BlockPlacement::new(IVec2::X, "plate", 0));
        let errors = registry.validate_ship(&overlapping).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("overlapping")),
            "{errors:?}"
        );

        let plate_only = vec![BlockPlacement::new(IVec2::ZERO, "plate", 0)];
        let errors = registry.validate_ship(&plate_only).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("core")), "{errors:?}");
    }
}
