# Block Shape

## Purpose

Block footprints are authored as one readable `ShapeSpec`, baked once at config load into
footprints and hitboxes for every placement rotation, with geometry-derived structural
connection and a `Visual` hierarchy that keeps colliders unscaled.

## Requirements

### Requirement: Block shapes are authored as a single ShapeSpec

A block config SHALL declare its footprint with one `ShapeSpec` value — `Square`, `Rect`, `Triangle`, or `Rows` — and SHALL NOT declare a separate size or voxel list. `Triangle` and `Rows` SHALL carry an `Xform { rotate: Rot90, flip_x: bool, flip_y: bool }` that defaults to identity.

#### Scenario: Square shorthand
- **WHEN** a block declares `Square(3)`
- **THEN** its footprint is the 3×3 grid of nine cells

#### Scenario: No duplicate geometry fields
- **WHEN** a block config is inspected
- **THEN** it contains one shape spec and no separate `size` or `voxels` field

### Requirement: Shapes are baked once at config load

Config loading SHALL convert each `ShapeSpec` into a `BakedShape` through `TryFrom`, which validates the spec and, for each of `R0`, `R90`, `R180`, `R270`, stores a `Footprint { cells, size }` and a `Hitbox`. `BlockDef` SHALL hold only the `BakedShape`, with private fields exposed as slices and values. Hot reload SHALL rebuild baked shapes with the registry. No system SHALL rasterize shapes at runtime.

#### Scenario: Footprint lookup does not allocate
- **WHEN** a system reads a block's cells for a placement rotation
- **THEN** it receives a `&[IVec2]` from the baked shape without rasterizing

#### Scenario: Hot reload refreshes baked shapes
- **WHEN** `block.ron` changes a block's shape and reloads
- **THEN** the rebuilt registry holds the new baked footprints and hitboxes

#### Scenario: Bounding box of a non-rectangular shape
- **WHEN** an L-shaped `Rows` footprint is baked
- **THEN** `size` is the bounding box of its cells and `cells` excludes the empty corner

### Requirement: Transforms apply in a fixed order

Base cells SHALL be transformed by the definition `Xform` (rotate counter-clockwise, then `flip_x`, then `flip_y`), normalized to min (0,0), then rotated by the placement `Rot90` counter-clockwise and normalized again. `size` SHALL be recomputed after each normalization.

#### Scenario: Four right-triangle corners
- **WHEN** `Triangle(Right, 3×3)` is baked with each definition rotation R0, R90, R180, R270
- **THEN** the four right-angle corner orientations result, each normalized to (0,0)

#### Scenario: Mirrored footprint
- **WHEN** a footprint has `flip_x: true`
- **THEN** its cells are mirrored horizontally and renormalized

#### Scenario: Placement rotation swaps size
- **WHEN** `Rect((3,5))` is placed with `Rot90::R90`
- **THEN** its footprint size is `(5,3)`

### Requirement: Triangles rasterize by center-in and must span their declared size

Triangle bases SHALL be `Right` and `Isosceles`. A cell SHALL be included only when its center satisfies the triangle's half-plane inequalities, evaluated with integer arithmetic, with no apex special-casing. Validation SHALL reject a triangle whose rasterized bounding box differs from its declared size.

#### Scenario: Right triangle staircase
- **WHEN** `Triangle(Right, 3×3)` is baked
- **THEN** its six cells form rows of widths 3, 2, and 1 from the base

#### Scenario: Isosceles triangle
- **WHEN** `Triangle(Isosceles, 5×3)` is baked
- **THEN** its cell rows have widths 5, 3, and 1 centered on the base

#### Scenario: Shrinking triangle rejected
- **WHEN** `Triangle(Isosceles, 4×3)` rasterizes to a 4×2 bounding box
- **THEN** config loading rejects it with an error naming the block

### Requirement: Rows declare masks with a fixed character set

`Rows` SHALL use `#` for filled and `.` for empty. A `Rows` spec SHALL be rejected if it contains any other character, if rows differ in length, if no `#` is present, or if its first or last row or column is entirely empty.

#### Scenario: Ring mask
- **WHEN** `Rows` declares `["###", "#.#", "###"]`
- **THEN** the footprint has eight cells and size `(3,3)`

#### Scenario: Empty border rejected
- **WHEN** `Rows` declares `["...", "##.", "###"]`
- **THEN** config loading rejects it with an error naming the block

### Requirement: Shape specs are validated

Config loading SHALL reject a shape spec with non-positive dimensions, malformed `Rows`, a declared-size mismatch, zero cells, more than one Manhattan-1 connected component, or a triangle whose convex hull cannot be built. Errors SHALL name the block key and produce a recoverable config error.

#### Scenario: Disconnected cells rejected
- **WHEN** a `Rows` spec describes two cell groups with no shared orthogonal edge
- **THEN** config loading rejects it with an error naming the block

### Requirement: Structural connection is derived from shared edges

Two cells of different blocks SHALL connect only when their coordinates differ by Manhattan distance 1. Diagonal contact SHALL NOT connect. No face mask SHALL exist in configuration or runtime types.

#### Scenario: Full edge connects
- **WHEN** a placed block's cell and an existing block's cell differ by `(1,0)` or `(0,1)`
- **THEN** the placement forms a structural connection

#### Scenario: Corner contact does not connect
- **WHEN** two cells touch only at a diagonal corner
- **THEN** no connection forms

### Requirement: Hitboxes match the shape

Baked hitboxes SHALL be a cuboid for `Square` and `Rect`, a convex polygon of the true triangle for `Triangle`, and a compound of per-cell cuboids for `Rows`, expressed in block-local space centered on the footprint bounding box for each placement rotation. Each block entity's sensor collider SHALL use its baked hitbox.

#### Scenario: Triangle corner is not hittable
- **WHEN** a projectile passes through the empty half of a right triangle block's bounding box
- **THEN** it does not collide with that block

### Requirement: Block visuals use a Visual child

A block entity SHALL carry placement translation, placement rotation, and the unscaled collider. Its textures and turrets SHALL be children of a `Visual` entity whose transform applies the definition rotation and the flips as signed scale. Art SHALL be authored once for the base shape, with alpha matching the true shape.

#### Scenario: One texture serves all corners
- **WHEN** four block defs reuse one right-triangle texture with different definition rotations
- **THEN** each renders in its own orientation and its collider has no negative scale

#### Scenario: Mirrored turret aims correctly
- **WHEN** a turret block with `flip_x: true` targets an enemy
- **THEN** its barrels point at the target in world space
