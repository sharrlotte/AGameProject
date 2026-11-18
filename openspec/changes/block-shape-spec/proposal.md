## Why

A block footprint is currently defined twice: `BlockShape.size` drives cell occupancy, collision, sprite sizing, fragment traversal and validation, while `BlockShape.voxels` drives only placement adjacency. Validation never requires voxels to fill `size`, so any non-rectangular shape silently occupies its full bounding rectangle — and authoring a 3×3 block takes nine near-identical RON lines whose `connectable_faces: 15` is redundant everywhere. `BlockPlacement.rotation` exists but is ignored everywhere, and every block hitbox is a bounding-box cuboid. We need one readable authoring format that supports squares, rectangles, and triangles, real placement rotation, and hitboxes that match the shape.

## What Changes

- **BREAKING** Replace `BlockShape { size: IVec2, voxels: Vec<BlockVoxel> }` in config with a `ShapeSpec` enum: `Square(u32)`, `Rect(UVec2)`, `Triangle { base: Right | Isosceles, size: UVec2, xform: Xform }`, `Rows { art: Vec<String>, xform: Xform }`.
- Bake each `ShapeSpec` once at config load into a runtime `BakedShape` holding, for each of the four placement rotations, a `Footprint { cells, size }` and a `Hitbox`. `BlockDef` stores only the baked form; RON stays the single authored source. Hot reload rebuilds the registry, so baked data cannot go stale.
- **BREAKING** `BlockPlacement.rotation: u8` becomes `Rot90 { R0, R90, R180, R270 }` and is applied at placement. Transform pipeline: spec → definition `Xform` (rotate, flip_x, flip_y) → normalize → placement `Rot90` → normalize.
- **BREAKING** Bump `SAVE_SCHEMA_VERSION` 4→5 and `DESIGN_SCHEMA_VERSION` 1→2. Old saves and designs are rejected by the existing version check. Starter designs are migrated.
- Delete `BlockVoxel`, `connectable_faces`, and `FACE_*`. Structural connection is derived from geometry: cells of different blocks connect only when they share a full orthogonal edge (Manhattan-1).
- Rasterize triangles with an integer center-in rule. Validation rejects any `Triangle` whose cells do not span its declared size, and any `Rows` with an empty border row or column. Also rejected: shapes with no cells, malformed `Rows`, and disconnected cells. Errors name the block key.
- Hitboxes: `Square`/`Rect` → cuboid, `Triangle` → exact convex polygon, `Rows` → compound of per-cell cuboids. Each block entity's sensor collider uses its baked hitbox. The ship physics collider becomes a compound of the per-block hitboxes of live blocks, replacing `voxels_from_points`.
- Rendering: block art is drawn once for the base (untransformed) shape. Its alpha matches the true shape (smooth triangle). A `Visual` child entity carries the signed scale (flip) and the definition's rotation. The block entity carries placement rotation and stays unscaled, so the collider never sees negative scale. Texture offsets and turret barrels inherit the transform.
- Editor: Q/E rotate the selected block counter-clockwise/clockwise. Bindings live in a new `keys` section of `setting.ron`. `update_placement_preview` runs only when the hovered cell, selected block, or rotation changes, and updates the preview instead of respawning it every frame.
- Migrate `block.ron` and starter designs, and document everything in `docs/`.

Out of scope: 3D extrusion, hollow/outline or equilateral triangles, per-cell connection overrides, placement-time flipping, automated art validation, and save migration from v4/v1.

## Capabilities

### New Capabilities

- `block-shape`: Shape authoring formats, definition `Xform`, placement `Rot90`, baked footprints and hitboxes, triangle rasterization, shape validation, geometry-derived connection, and the `Visual` rendering hierarchy.

### Modified Capabilities

- `config-validation`: Replace the "shape `size > 0`, voxels within size, no duplicate voxels" rule with shape-spec validation, including the declared-size and Rows-border rules.
- `ship-grid`: Occupied cells come from the baked footprint for the placement rotation. Connectivity uses Manhattan-1. The ship collider is a compound of baked block hitboxes.
- `world-save`: `BlockPlacement.rotation` is `Rot90`; world save v5 and design v2.
- `editor-ui`: Q/E rotation from `setting.ron` keys, and a change-driven placement preview.

## Impact

- `core/src/content/block/{component.rs, placement.rs, system.rs}`, `core/src/config/block.rs`, `core/assets/config/base/block.ron`, `core/assets/config/setting.ron`.
- `core/src/content/ship/{grid.rs, mutation.rs, component.rs}`, `core/src/scene/editor/{grid.rs, input.rs, mod.rs, ship_list.rs}`, `core/src/scene/game/mod.rs`.
- `core/src/save/{battle,design}/component.rs` version constants; `core/assets/config/starter/*.ron`.
- Specs: new `block-shape`; deltas for `config-validation`, `ship-grid`, `world-save`, `editor-ui`.
- Docs: `docs/config.md`, `docs/scenes.md`, `docs/simulation.md`, and save docs.
