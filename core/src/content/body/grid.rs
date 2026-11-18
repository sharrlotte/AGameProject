use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::content::block::{BlockDef, BlockPlacement, BlockRegistry, GridBlock};

#[derive(Component, Debug, Default)]
pub struct BlockGrid {
    pub blocks: HashMap<Uuid, GridBlock>,
    pub cells: HashMap<IVec2, Uuid>,
}

impl BlockGrid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_block(&mut self, placement: BlockPlacement, entity: Entity, def: &BlockDef) {
        let footprint = def.shape.get(placement.rotation).0;

        for offset in footprint.cells() {
            self.cells
                .insert(placement.position + *offset, placement.id);
        }
        self.blocks
            .insert(placement.id, GridBlock { placement, entity });
    }

    pub fn set_entity(&mut self, id: Uuid, entity: Entity) {
        if let Some(block) = self.blocks.get_mut(&id) {
            block.entity = entity;
        }
    }

    pub fn block(&self, id: Uuid) -> Option<&GridBlock> {
        self.blocks.get(&id)
    }

    pub fn block_at(&self, cell: IVec2) -> Option<&GridBlock> {
        let id = self.cells.get(&cell)?;
        self.blocks.get(id)
    }

    pub fn solid_neighbor(
        &self,
        registry: &BlockRegistry,
        cell: IVec2,
        dir: IVec2,
    ) -> Option<Uuid> {
        let source_id = *self.cells.get(&cell)?;
        let neighbor_id = *self.cells.get(&(cell + dir))?;
        if source_id == neighbor_id {
            return None;
        }

        let source = self.blocks.get(&source_id)?;
        let neighbor = self.blocks.get(&neighbor_id)?;
        let source_def = registry.get_symbol(&source.placement.block_type)?;
        let neighbor_def = registry.get_symbol(&neighbor.placement.block_type)?;

        let source_offset = cell - source.placement.position;
        let neighbor_offset = cell + dir - neighbor.placement.position;
        if source_def.solid_edge(source.placement.rotation, source_offset, dir)
            && neighbor_def.solid_edge(neighbor.placement.rotation, neighbor_offset, -dir)
        {
            Some(neighbor_id)
        } else {
            None
        }
    }

    pub fn remove_block(&mut self, id: Uuid) -> Option<GridBlock> {
        self.cells.retain(|_, cell_id| *cell_id != id);
        self.blocks.remove(&id)
    }

    pub fn blocks(&self) -> impl Iterator<Item = &GridBlock> {
        self.blocks.values()
    }

    pub fn components(&self, registry: &BlockRegistry) -> Vec<Vec<Uuid>> {
        let mut visited: HashSet<Uuid> = HashSet::new();
        let mut components = Vec::new();

        for id in self.blocks.keys() {
            if !visited.insert(*id) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = vec![*id];

            while let Some(current) = queue.pop() {
                component.push(current);

                let Some(block) = self.block(current) else {
                    continue;
                };
                let Some(def) = registry.get_symbol(&block.placement.block_type) else {
                    continue;
                };

                let footprint = def.shape.get(block.placement.rotation).0;
                for offset in footprint.cells() {
                    let cell = block.placement.position + *offset;
                    for direction in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                        if let Some(neighbor) = self.solid_neighbor(registry, cell, direction)
                            && visited.insert(neighbor)
                        {
                            queue.push(neighbor);
                        }
                    }
                }
            }

            components.push(component);
        }

        components
    }

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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::GRID_SIZE;
    use crate::content::block::{BakedShape, BlockDef, BlockRegistry, Rot90, ShapeSpec, Xform};

    impl BlockGrid {
        fn entity_at(&self, cell: IVec2) -> Option<Entity> {
            let id = self.cells.get(&cell)?;
            self.blocks.get(id).map(|block| block.entity)
        }

        fn occupied_cells(&self) -> impl Iterator<Item = (&IVec2, &Uuid)> {
            self.cells.iter()
        }
    }

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        def_shape(block_type, group, mass, health, ShapeSpec::Square(1))
    }

    fn def_shape(
        block_type: &str,
        group: &str,
        mass: f32,
        health: f32,
        shape: ShapeSpec,
    ) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BakedShape::try_from(&shape).expect("test shape bakes"),
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
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
        ]
    }

    #[test]
    fn cells_follow_shapes_and_removals() {
        let registry = registry();
        let mut grid = BlockGrid::new();
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
        let mut grid = BlockGrid::new();
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
    fn core_loss_yields_no_build_result() {
        let registry = registry();
        let mut grid = BlockGrid::new();
        grid.rebuild_from_layout(&layout(), &registry);

        let core_id = grid.block_at(IVec2::ZERO).unwrap().placement.id;
        grid.remove_block(core_id);

        assert!(registry.core_of(&grid).is_none());
        assert!(registry.stats_from_grid(&grid).is_none());
        assert!(registry.build_grid_result(&grid).is_none());
    }

    #[test]
    fn validate_rejects_overlap_and_missing_core() {
        let registry = registry();

        let mut overlapping = layout();
        overlapping.push(BlockPlacement::new(IVec2::X, "plate", Rot90::R0));
        let errors = registry.validate_ship(&overlapping).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("overlapping")),
            "{errors:?}"
        );

        let plate_only = vec![BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0)];
        let errors = registry.validate_ship(&plate_only).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("core")), "{errors:?}");
    }

    #[test]
    fn rotated_and_masked_footprints_occupy_only_baked_cells() {
        let mut registry = BlockRegistry::default();
        registry.register(
            "core",
            def_shape(
                "core",
                "core",
                10.0,
                100.0,
                ShapeSpec::Rect(UVec2::new(1, 3)),
            ),
        );
        registry.register(
            "ring",
            def_shape(
                "ring",
                "structure",
                5.0,
                50.0,
                ShapeSpec::Rows {
                    art: vec!["###".to_string(), "#.#".to_string(), "###".to_string()],
                    xform: Xform::default(),
                },
            ),
        );

        let mut grid = BlockGrid::new();
        let rotated = BlockPlacement::new(IVec2::new(10, 10), "core", Rot90::R90);
        let ring = BlockPlacement::new(IVec2::new(-5, -5), "ring", Rot90::R0);
        grid.insert_block(rotated.clone(), Entity::PLACEHOLDER, registry.get_symbol("core").unwrap());
        grid.insert_block(ring.clone(), Entity::PLACEHOLDER, registry.get_symbol("ring").unwrap());

        for x in 10..13 {
            assert!(grid.block_at(IVec2::new(x, 10)).is_some());
            assert!(grid.block_at(IVec2::new(x, 11)).is_none());
        }
        assert!(grid.block_at(IVec2::new(-5, -5)).is_some());
        assert!(grid.block_at(IVec2::new(-4, -4)).is_none());
    }
}
