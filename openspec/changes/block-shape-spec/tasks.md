## 1. Shape types and baking

- [x] 1.1 Define `Rot90`, `Xform`, `TriangleBase`, `ShapeSpec` (config) and `Footprint`, `Hitbox`, `BakedShape` (runtime, private fields) in `content/block/component.rs`
- [x] 1.2 Implement `Square`, `Rect`, and `Rows` cell generation, and `Right`/`Isosceles` integer center-in rasterization with no apex special-casing
- [x] 1.3 Implement the pipeline: definition Xform (rotate → flip_x → flip_y) → normalize → placement Rot90 → normalize
- [x] 1.4 Implement `TryFrom<&ShapeSpec> for BakedShape`, baking footprints and hitboxes for all four rotations
- [x] 1.5 Bake hitboxes: cuboid for Square/Rect, convex polygon for Triangle, per-cell cells for Rows
- [x] 1.6 Unit tests: square/rect cells, all four right-triangle corners, isosceles directions, flips, normalization, rotated size, rotated hitbox vertices, non-rectangular masks

## 2. Config schema and validation

- [x] 2.1 Change `BlockDefConfig.shape` to `ShapeSpec` and bake in `into_def`
- [x] 2.2 Validate positive dimensions, Rows charset/rectangularity/no empty border row or column, triangle bounding box equal to declared size, ≥1 cell, single Manhattan-1 component, valid convex hull
- [x] 2.3 Route failures to `ConfigState::Error` with the block key
- [x] 2.4 Tests: disconnected cells, ragged rows, invalid characters, empty shape, empty Rows border, `Isosceles 4×3` rejected

## 3. Placement rotation and saves

- [x] 3.1 Change `BlockPlacement.rotation` to `Rot90`
- [x] 3.2 Bump `SAVE_SCHEMA_VERSION` to 5 and `DESIGN_SCHEMA_VERSION` to 2
- [x] 3.3 Migrate `core/assets/config/starter/*.ron` to version 2 and `rotation: R0`
- [x] 3.4 Update save tests for the new versions and a rotated-block round-trip

## 4. Consumer migration

- [x] 4.1 `BlockDef::{get_block_size, get_pixel_size, to_transform}` take `Rot90`; migrate `BlockRegistry::core_of` and layout validation
- [x] 4.2 `ShipGrid::{insert_block, refresh_cells}` use the rotated footprint
- [x] 4.3 Fragment BFS in `ship/mutation.rs` uses the rotated footprint
- [x] 4.4 `check_placement_validity` uses the rotated footprint and Manhattan-1 connection, keeping empty-grid acceptance
- [x] 4.5 Migrate `scene/editor/{input,ship_list}.rs`, `scene/game/mod.rs`, and every remaining grep hit (icons use `Rot90::R0`)
- [x] 4.6 Grid tests: rotated and non-rectangular occupancy, no phantom cells

## 5. Rendering and collision

- [x] 5.1 Add the `Visual` marker and spawn block textures/turrets under it with signed scale and definition rotation; the block entity carries placement rotation
- [x] 5.2 Block sensor collider built from the baked hitbox
- [x] 5.3 Replace `build_ship_voxel_collider_2d` with a compound of live blocks' baked hitboxes
- [x] 5.4 Tests: turret world-space aim under `flip_x`/`flip_y`, collider shape per variant, ship collider after fragment removal

## 6. Editor

- [x] 6.1 Add `keys: (rotate_ccw, rotate_cw)` to the settings type and `setting.ron`, merged with defaults (Q/E)
- [x] 6.2 Add `rotation: Rot90` to `EditorState`; Q/E rotate it; apply to anchor, preview, validity, and placement
- [x] 6.3 Rewrite `update_placement_preview` to run only on change and update one persistent preview entity

## 7. Removal

- [x] 7.1 Delete `BlockShape`, `BlockVoxel`, `connectable_faces`, `FACE_*`
- [x] 7.2 Confirm no remaining references, dead code, or unused imports

## 8. Data and documentation

- [x] 8.1 Migrate every `block.ron` shape to `Square`, `Rect`, `Triangle`, or `Rows`
- [x] 8.2 `docs/config.md`: shape format, transform order, validation, art convention (base shape, true-shape alpha), `keys` setting
- [x] 8.3 `docs/scenes.md`: placement rule, rotation input, preview; `docs/simulation.md`: baked footprints, hitboxes, ship collider; save docs: v5/v2 and `Rot90`

## 9. Verification

- [x] 9.1 Run targeted `cargo test` for block, ship, save, and config modules
- [x] 9.2 Run `cargo dev` in the background; smoke-test placing and rotating square, rect, triangle, and Rows blocks, check hit detection on triangle corners, then terminate
