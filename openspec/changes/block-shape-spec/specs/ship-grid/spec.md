## MODIFIED Requirements

### Requirement: ShipGrid is the single source of truth for placement and identity

The system SHALL store every block placement in `ShipGrid` as a `GridBlock { placement: BlockPlacement, entity: Entity }` keyed by block id, with `cells: HashMap<IVec2, Uuid>` mapping every occupied cell to that id, and SHALL NOT keep a second block list or duplicate placement fields on block entities. Occupied cells SHALL be `placement.position` plus the baked footprint cells for `placement.rotation`.

#### Scenario: Editor placement and runtime spawn agree
- **WHEN** the editor places a block and a ship spawns from a design
- **THEN** both produce the same `BlockPlacement` fields and the same `GridBlock` grid entry

#### Scenario: Destroyed block leaves the grid
- **WHEN** a block is destroyed
- **THEN** none of its occupied cells resolve to its id in `ShipGrid.cells` and no block query can retrieve it

#### Scenario: Non-rectangular footprint occupies only its cells
- **WHEN** a block with a hole or triangular footprint is inserted
- **THEN** only its baked cells map to its id and the empty region of its bounding box remains unoccupied

#### Scenario: Rotated footprint occupies rotated cells
- **WHEN** a `Rect((1,3))` block is inserted with `Rot90::R90`
- **THEN** it occupies three cells along the x axis

### Requirement: Core-less fragments are removed from grid and collider

After a mutation, the system SHALL compute connected components from `ShipGrid.cells` using Manhattan-1 adjacency of baked footprint cells, find the component containing the core, remove every block in a core-less fragment from the grid, mark those block entities `Death`, and rebuild the ship collider as a compound of the baked hitboxes of live blocks only.

#### Scenario: Detached section becomes a fragment
- **WHEN** a block connecting a side section to the core is destroyed
- **THEN** the detached section is removed from the grid, its entities are marked `Death`, and the collider no longer contains its hitboxes

#### Scenario: Core destruction ends the ship
- **WHEN** the core is destroyed
- **THEN** the ship is marked `Death`, no core-less component survives as a ship, and collider rebuild is skipped

#### Scenario: Ship collider follows triangle edges
- **WHEN** a ship with a triangle block is rebuilt
- **THEN** its collider includes that block's triangle polygon, not a voxel staircase

### Requirement: Spawn and editor validate ship layouts

Both spawn and editor paths SHALL build the grid through `ShipGrid::rebuild_from_layout` and SHALL validate exactly one core, no overlapping cells, all occupied cells derived from baked footprints for each placement rotation, connected components using Manhattan-1 adjacency with no disconnected non-core components at spawn, no empty turret barrels, and valid power/storage values.

#### Scenario: Overlapping cell is reported
- **WHEN** a design places two blocks in the same cell
- **THEN** validation returns an error naming the cell

#### Scenario: Disconnected hull is rejected
- **WHEN** a design contains a disconnected hull section at spawn
- **THEN** validation rejects the spawn

#### Scenario: Diagonal-only hull is disconnected
- **WHEN** two sections of a hull touch only at diagonal corners
- **THEN** validation treats them as separate components and rejects non-core fragments

### Requirement: Ship grid invariants have automated tests

Unit tests SHALL cover multi-cell insert/remove, overlap rejection, negative coordinates, `rebuild_from_layout`, cell counts versus baked footprints, rotated and non-rectangular footprints, stats from grid, fragment removal, core death, bounds updates, and compound collider contents. An integration test SHALL spawn a ship, destroy a mid-ship block, and assert grid, collider, and bounds.

#### Scenario: Grid test suite runs
- **WHEN** `cargo test` runs
- **THEN** the grid, stats, and mutator suites pass and assert no phantom cells
