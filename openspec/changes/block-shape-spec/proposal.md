## Why

A block footprint is currently defined twice: `BlockShape.size` drives cell occupancy, collision, sprite sizing, fragment traversal and validation, while `BlockShape.voxels` drives only placement adjacency. Validation never requires voxels to fill `size`, so any non-rectangular shape silently occupies its full bounding rectangle — and authoring a 3×3 block takes nine near-identical RON lines whose `connectable_faces: 15` is redundant everywhere. We need one readable authoring format that supports squares, rectangles, and triangles without duplicating geometry.

## What Changes

- **BREAKING** Replace `BlockShape { size: IVec2, voxels: Vec<BlockVoxel> }` with a `ShapeSpec` enum: `Square(u32)`, `Rect(UVec2)`, `Triangle { base: Right | Isosceles, size: UVec2, xform: Xform }`, `Rows { art: Vec<String>, xform: Xform }`.
- Derive `ShapeSpec::cells() -> Vec<IVec2>` and `ShapeSpec::size() -> IVec2` on demand. No stored/baked cell list and no stored `size`.
- Migrate every cell/size consumer to the derived methods: grid insert/refresh, placement validity, collision, fragment traversal, overlap validation, core offset, sprite sizing, and bounds.
- Delete `BlockVoxel` and `connectable_faces`. Structural connection is derived from geometry: cells of different blocks connect only when they share a full orthogonal edge (Manhattan-1); diagonal corner contact never connects.
- Rasterize triangles with a center-in rule; apply `rotate` (R0/R90/R180/R270) and `flip_x`/`flip_y` to generated cells, then normalize to min=(0,0).
- Validate shape specs at config load: valid dimensions, well-formed `Rows`, at least one cell, and a single Manhattan-1 connected component. Errors name the block key.
- Migrate `block.ron` shapes and document the format plus the alpha-matched-art convention in `docs/config.md`. No automated art/mask drift check.

Out of scope: 3D extrusion, hollow/outline or equilateral triangles, per-cell connection overrides, baking/caching derived cells, automated art validation, and rendering changes beyond alpha-matched textures.

## Capabilities

### New Capabilities

- `block-shape`: Shape authoring formats, rotate/flip transforms, derived cells and bounding box, triangle rasterization, shape validation, and geometry-derived connection semantics.

### Modified Capabilities

- `config-validation`: Replace the "shape `size > 0`, voxels within size, no duplicate voxels" rule with validation of the new shape spec and its derived cells.
- `ship-grid`: Occupied cells and connected components derive from `ShapeSpec::cells()` with Manhattan-1 adjacency instead of a stored `size` rectangle.

## Impact

- `core/src/content/block/component.rs` (`BlockShape`, `BlockVoxel`, `BlockDef`, registry validation and helpers), `core/src/config/block.rs`, `core/assets/config/base/block.ron`.
- `core/src/content/ship/grid.rs`, `core/src/content/ship/mutation.rs`, `core/src/scene/editor/grid.rs`, and any other cell/size iteration site.
- Remove all `connectable_faces` and `FACE_*` constants and their uses.
- Specs: new `block-shape`; delta specs for `config-validation` and `ship-grid`.
- Docs: `docs/config.md` shape section, `docs/scenes.md` placement rule, `docs/simulation.md` multi-cell note.
- Save schema is unchanged (`BlockPlacement` only); no new runtime components or systems.
