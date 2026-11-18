# block-grid Specification

## Purpose
TBD - created by archiving change debris-bodies. Update Purpose after archive.
## Requirements
### Requirement: BlockGrid is the single source of truth for placement and identity

The system SHALL store every block placement in `BlockGrid` as a `GridBlock { placement: BlockPlacement, entity: Entity }` keyed by block id, with `cells: HashMap<IVec2, Uuid>` mapping every occupied cell to that id, and SHALL NOT keep a second block list or duplicate placement fields on block entities. Occupied cells SHALL be `placement.position` plus the baked footprint cells for `placement.rotation`.

#### Scenario: Editor placement and runtime spawn agree
- **WHEN** the editor places a block and a ship spawns from a design
- **THEN** both produce the same `BlockPlacement` fields and the same `GridBlock` grid entry

#### Scenario: Destroyed block leaves the grid
- **WHEN** a block is destroyed
- **THEN** none of its occupied cells resolve to its id in `BlockGrid.cells` and no block query can retrieve it

#### Scenario: Non-rectangular footprint occupies only its cells
- **WHEN** a block with a hole or triangular footprint is inserted
- **THEN** only its baked cells map to its id and the empty region of its bounding box remains unoccupied

#### Scenario: Rotated footprint occupies rotated cells
- **WHEN** a `Rect((1,3))` block is inserted with `Rot90::R90`
- **THEN** it occupies three cells along the x axis

### Requirement: Block entities carry only identity and runtime state

A spawned block entity SHALL carry `BlockRef { id }` and `BlockOwner { body }`; block type, position, and rotation SHALL be read from `BlockGrid` via `BlockRef`. `Block`, `BlockData`, `BlockState`, and `Blocks` SHALL NOT exist.

#### Scenario: Placement is read from the grid
- **WHEN** a system needs a block's type, position, or rotation
- **THEN** it resolves the `BlockRef` in `BlockGrid` and the entity has no placement component

#### Scenario: Block identity components are unique
- **WHEN** a block entity is spawned
- **THEN** it has exactly one `BlockRef` and one `BlockOwner`

#### Scenario: Block owner follows a grid move
- **WHEN** a block is detached from a ship and becomes part of a debris body
- **THEN** its `BlockOwner.body` names the debris body, not the ship

### Requirement: Block destruction goes through BodyMutation

The system SHALL expose `BodyMutation::DestroyBlock { body, block_id }` and apply mutations once per fixed tick through `apply_body_mutations`; no system SHALL mutate a live body's `BlockGrid` directly.

#### Scenario: Destruction pipeline
- **WHEN** a block is destroyed on a ship
- **THEN** the mutation removes every occupied cell, recomputes connected components, detaches non-core components as debris, updates stats and bounds, rebuilds the ship collider, and emits `ShipChanged`

#### Scenario: Destruction pipeline on debris
- **WHEN** a block is destroyed on a debris body
- **THEN** the mutation removes every occupied cell, recomputes components, splits the remaining blocks into debris bodies, and rebuilds each body's collider without touching `ShipStats` or `BoundingBox`

### Requirement: Detached components leave the ship

After a mutation, the system SHALL compute connected components from `BlockGrid.cells` using Manhattan-1 adjacency of baked footprint cells, find the component containing the core, and detach every other component into an independent debris body. The ship grid and ship collider SHALL contain only the core component.

#### Scenario: Detached section becomes debris
- **WHEN** a block connecting a side section to the core is destroyed
- **THEN** the detached section leaves the ship grid, becomes a debris body, and the ship collider no longer contains its hitboxes

#### Scenario: Core destruction ends the ship
- **WHEN** the core is destroyed
- **THEN** the ship entity despawns, `ShipDestroyed` is emitted, and every surviving component becomes a debris body

#### Scenario: Ship collider follows triangle edges
- **WHEN** a ship with a triangle block is rebuilt
- **THEN** its collider includes that block's triangle polygon, not a voxel staircase

#### Scenario: No block belongs to two bodies
- **WHEN** a component is detached
- **THEN** every block in it exists in exactly one `BlockGrid` and has exactly one `BlockOwner`

### Requirement: Ship identity and stats derive from the core and live grid

`Ship.id` SHALL equal the core block id and `Ship.core_position` SHALL be the core block center in ship-local space. `ShipStats` SHALL be the single stats component and SHALL include mass, max/min range, health/max_health, power aggregates, cargo aggregates, thrust power, and rotate speed. `BoundingBox` SHALL initialize from `i32::MAX`/`i32::MIN` and SHALL be recomputed on every ship mutation.

#### Scenario: Negative coordinates are included in bounds
- **WHEN** a ship spawns with blocks at negative coordinates
- **THEN** `BoundingBox` includes those cells and no coordinate clamps to zero

#### Scenario: No turret means zero range
- **WHEN** a ship has no turret blocks
- **THEN** `ShipStats.max_range` and `ShipStats.min_range` are `0.0`

#### Scenario: Multiple cores are rejected
- **WHEN** a layout contains more than one core block
- **THEN** spawn validation rejects the layout and names the conflict

#### Scenario: Health cache follows block health
- **WHEN** any block's current health changes
- **THEN** `ShipStats.health` equals the sum of current block health on that ship

#### Scenario: Debris does not change ship stats
- **WHEN** a component is detached or a debris block is destroyed
- **THEN** no `ShipStats` or `BoundingBox` outside the debris body changes

### Requirement: ShipStats replaces overlapping stat types

The system SHALL use `ShipStats` as the only ship-level stat component; `ComputedShipStats`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipHealth`, `ShipPower`, and a separate `ShipCargo` stat split SHALL NOT exist.

#### Scenario: HUD and AI read one health value
- **WHEN** the HUD or AI needs ship HP
- **THEN** it reads `ShipStats.health`

#### Scenario: Aggregates update together
- **WHEN** storage, power, or thruster blocks change
- **THEN** the corresponding `ShipStats` fields update through the same grid recomputation

### Requirement: Spawn and editor validate ship layouts

Both spawn and editor paths SHALL build the grid through `BlockGrid::rebuild_from_layout` and SHALL validate exactly one core, no overlapping cells, all occupied cells derived from baked footprints for each placement rotation, connected components using Manhattan-1 adjacency with no disconnected non-core components at spawn, no empty turret barrels, and valid power/storage values.

#### Scenario: Overlapping cell is reported
- **WHEN** a design places two blocks in the same cell
- **THEN** validation returns an error naming the cell

#### Scenario: Disconnected hull is rejected
- **WHEN** a design contains a disconnected hull section at spawn
- **THEN** validation rejects the spawn

### Requirement: Block grid invariants have automated tests

Unit tests SHALL cover multi-cell insert/remove, overlap rejection, negative coordinates, `rebuild_from_layout`, cell counts versus block shapes, stats from grid, component detection, detached-component spawning, core death, bounds updates, and collider cell sets. An integration test SHALL spawn a ship, destroy a mid-ship block, and assert grid, collider, bounds, and the spawned debris body.

#### Scenario: Grid test suite runs
- **WHEN** `cargo test` runs
- **THEN** the grid, stats, and mutation suites pass and assert no phantom cells and no block in two grids

