## Context

Today `BlockShape { size: IVec2, voxels: Vec<BlockVoxel> }` encodes geometry twice. `size` is authoritative for cell occupancy (`ShipGrid::insert_block`, `refresh_cells`, fragment traversal, overlap validation, sprite sizing, core offset, bounds), while `voxels` is authoritative only for placement adjacency (`check_placement_validity`). Each `BlockVoxel` carries `connectable_faces: u8` (always `15` in shipped data). `BlockPlacement.rotation: u8` is stored in saves but ignored (`_rotation`). Every block sensor collider is `Collider::cuboid(half_size)`, and the ship collider is `Collider::voxels_from_points` over occupied cells. `update_placement_preview` despawns and respawns the preview entity every frame.

The registry is rebuilt from scratch on config hot reload (`config/block.rs`), and config→runtime conversion is an established pattern (`TurretComponentConfig` → `TurretComponent`, `AmmoConfig` → `Ammo`). Saves and designs are versioned RON documents that reject version mismatches.

Constraints from `AGENTS.md`: single source of truth; SOLID and modular; no `unwrap`/`unsafe`; validate and fail loud; RON config with hot reload; systems run only when needed; Bevy 0.20 idioms.

## Goals / Non-Goals

**Goals:**
- One readable authoring format for square, rectangle, and triangle footprints.
- Geometry computed once at config load; zero allocation and no rasterization in hot paths.
- Working placement rotation for all blocks, with a single texture per base shape.
- Geometry-derived structural connection (Manhattan-1).
- Hitboxes and ship collider that match the visible shape.
- A change-driven editor preview.

**Non-Goals:**
- 3D extrusion, hollow/outline or equilateral triangles, generic polygons, CSG.
- Placement-time flipping, per-cell connection overrides, automated art/mask checks.
- Migration of v4 world saves or v1 designs.

## Decisions

**D1 — `ShapeSpec` is the config type.** Variants: `Square(u32)`, `Rect(UVec2)`, `Triangle { base: TriangleBase, size: UVec2, xform: Xform }`, `Rows { art: Vec<String>, xform: Xform }`; `TriangleBase` is `Right | Isosceles`; `Xform { rotate: Rot90, flip_x: bool, flip_y: bool }` with `#[serde(default)]`. `BlockDefConfig.shape: ShapeSpec`. `ShapeSpec` never appears on runtime types.

**D2 — Bake at config load.** `impl TryFrom<&ShapeSpec> for BakedShape` validates, rasterizes, and transforms in one step. Validation and baking are the same operation, so an invalid shape can never exist at runtime.

```rust
pub struct Footprint { cells: Box<[IVec2]>, size: IVec2 }
pub enum Hitbox { Cuboid(Vec2), Polygon(Box<[Vec2]>), Cells(Box<[IVec2]>) }
pub struct BakedShape { rotations: [(Footprint, Hitbox); 4], xform: Xform }
impl BakedShape { pub fn get(&self, rot: Rot90) -> (&Footprint, &Hitbox) }
```

Fields are private; access is through `&[IVec2]`, `IVec2`, and `&Hitbox`. `BlockDef.shape: BakedShape`. RON remains the only authored source; the baked form is derived and read-only. Hot reload rebuilds the registry, so the baked form cannot go stale. Alternative: derive on every call. Rejected because fragment BFS runs on every block destruction and would rasterize once per reachable block.

**D3 — Variant set chosen for readability.** Considered: polygon outline, ASCII rows only, width skyline, half-plane DSL, CSG. The hybrid gives one-line common cases plus a `Rows` escape hatch.

**D4 — Transform pipeline.** Base cells → definition `Xform` (rotate CCW, then `flip_x`, then `flip_y`) → normalize to min=(0,0) → placement `Rot90` (CCW) → normalize. The order is fixed because rotate and flip do not commute. All four placement rotations are baked. A placed block's world cells are `placement.position + footprint.cells()`, where `footprint = def.shape.get(placement.rotation).0`. `size` is recomputed after each normalization, so `Rect(3,5)` at R90 has size `(5,3)`.

**D5 — Center-in rasterization with a declared-size check.** A cell is included when its center satisfies the triangle's half-plane inequalities, using integer cross-multiplication. `Right(w,h)`: `(2x+1)·h + (2y+1)·w <= 2·w·h`. `Isosceles(w,h)`: base `(0,0)-(w,0)`, apex `(w/2,h)`, center between both sloped edges. No apex special-casing. Validation rejects a triangle when its rasterized bounding box differs from the declared `size` (e.g. `Isosceles 4×3` yields 4×2 and is rejected). This keeps art and size in agreement.

**D6 — `Rows` format.** `#` = filled, `.` = empty; rows of equal length; at least one `#`; any other character is an error. A fully empty first/last row or first/last column is rejected, matching D5's declared-size rule.

**D7 — Connection from geometry.** Delete `BlockVoxel`, `connectable_faces`, `FACE_*`. Cells of different blocks connect only at Manhattan distance 1. `check_placement_validity` uses the rotated footprint, keeps the empty-grid acceptance rule, and replaces face-mask matching with this test. Within-block connectivity is guaranteed by validation.

**D8 — Validation at config load.** Positive dimensions, well-formed `Rows` (D6), declared-size match (D5), at least one cell, one Manhattan-1 component. Errors name the block key and flow through `ConfigState::Error`.

**D9 — Hitboxes.** Baked per rotation, in block-local pixel space centered on the footprint's bounding box. `Square`/`Rect` → `Hitbox::Cuboid(half_extents)`. `Triangle` → `Hitbox::Polygon` holding the three transformed vertices of the true triangle, built with `Collider::convex_hull`; a failed hull fails config validation. `Rows` → `Hitbox::Cells`, built as a compound of per-cell cuboids. The block sensor collider uses `def.shape.get(rot).1`.

**D10 — Ship collider.** The ship physics collider is `Collider::compound` of each live block's baked hitbox, positioned at the block's footprint center. It replaces `voxels_from_points` and is rebuilt at the same points as today (spawn, mutation). `ColliderMassProperties::Mass(stats.mass)` is unchanged.

**D11 — Art and the `Visual` hierarchy.** Art is drawn once for the base, untransformed shape, and its alpha matches the true shape (smooth triangle edges). Cells are only for grid occupancy and connection.

```
block entity   Transform(translation = footprint center, rotation = placement Rot90)   Collider (baked hitbox, unscaled)
 └─ Visual     Transform(scale = (±1, ±1, 1), rotation = definition Xform.rotate)
     ├─ Sprite children (Texture features, at visual.offset)
     └─ Turret children (offset, barrels)
```

`Visual` is a marker component. Flips use signed scale on `Visual` only, so offsets and barrels mirror automatically and no collider is ever under negative scale. The sprite `custom_size` is the base shape's size before any transform. Turret aiming must produce correct world-space angles under a mirrored parent; this is pinned by tests.

**D12 — Placement rotation and saves.** `BlockPlacement.rotation: Rot90`, serialized as the enum (`rotation: R90`). `SAVE_SCHEMA_VERSION` 4→5, `DESIGN_SCHEMA_VERSION` 1→2. Older files are rejected by the existing version check and left untouched on disk. Starter designs in `core/assets/config/starter/` are migrated to v2.

**D13 — Editor rotation input.** `setting.ron` gains `keys: (rotate_ccw: KeyQ, rotate_cw: KeyE)`, merged with defaults per the existing settings rule. `EditorState` holds `rotation: Rot90`, which is applied to preview, validity, and placement. The editor anchor uses the rotated size (`size / 2`).

**D14 — Change-driven preview.** `update_placement_preview` runs only when the hovered grid cell, selected block, rotation, or grid changes (a run condition on `EditorState` change detection). It keeps one persistent preview entity and mutates its transform, tint, and visual children instead of despawning it every frame. Children are rebuilt only when the selected block changes.

**D15 — Consumer migration.** Every `def.shape.size` / `def.shape.voxels` read becomes a lookup on `def.shape.get(placement.rotation)`. Sites: `BlockDef::{get_block_size, get_pixel_size, to_transform}` (these gain a `Rot90` parameter), `BlockRegistry::{core_of, layout validation, build_ship_voxel_collider_2d}`, `ShipGrid::{insert_block, refresh_cells}`, fragment BFS in `ship/mutation.rs`, `spawn_block_entity`, `scene/editor/{grid,input,ship_list}.rs`, and `scene/game/mod.rs`. UI icons (`ship_list`, game HUD) use `Rot90::R0`.

## Risks / Trade-offs

- [Baked memory] → four footprints and hitboxes per block type, each a handful of cells; negligible.
- [Rotate/flip order is non-commutative] → fixed order plus unit tests for each step.
- [Old saves and designs become unloadable] → accepted by the user. Files stay on disk. The starter designs are re-seeded only when the designs directory is empty, so the v1 starter copies already on disk will also fail to load.
- [Turret aiming under a mirrored `Visual`] → covered by tests that assert world-space aim with `flip_x` and `flip_y`.
- [Compound ship collider is heavier than voxels] → one shape per block (cells only for `Rows`). It is rebuilt only on mutation, as today.
- [Cell mask vs smooth hitbox differ at triangle edges] → intended. Cells govern grid and connection; hitboxes govern physics and hits.
- [No automated art check] → convention documented.

## Migration Plan

1. Add `Rot90`, `Xform`, `TriangleBase`, `ShapeSpec`, `Footprint`, `Hitbox`, `BakedShape`, and `TryFrom` baking with unit tests.
2. Switch `BlockDefConfig.shape` to `ShapeSpec`; bake in `into_def`; route errors with the block key.
3. Change `BlockPlacement.rotation` to `Rot90`; bump save/design versions; migrate starter designs.
4. Migrate all consumers; add the `Visual` hierarchy and baked hitbox colliders; replace the ship collider.
5. Add `keys` to `setting.ron`, editor rotation, and the change-driven preview.
6. Delete `BlockShape`, `BlockVoxel`, `connectable_faces`, `FACE_*`.
7. Migrate `block.ron`; update specs and docs.
8. Gate: targeted `cargo test`, then `cargo dev` in the background. Rollback is a straight revert.

## Open Questions

None. All forks were settled with the user, including baking, placement rotation, the declared-size rule, hitboxes, the ship collider, art, the `Visual` hierarchy, save versioning, key bindings, and the preview rewrite.
