## ADDED Requirements

### Requirement: ShipGrid is the single source of truth for placement and identity

The system SHALL store every block placement in `ShipGrid` as a `GridBlock { placement: BlockPlacement, entity: Entity }` keyed by block id, with `cells: HashMap<IVec2, Uuid>` mapping every occupied cell to that id, and SHALL NOT keep a second block list or duplicate placement fields on block entities.

#### Scenario: Editor placement and runtime spawn agree
- **WHEN** the editor places a block and a ship spawns from a design
- **THEN** both produce the same `BlockPlacement` fields and the same `GridBlock` grid entry

#### Scenario: Destroyed block leaves the grid
- **WHEN** a block is destroyed
- **THEN** none of its occupied cells resolve to its id in `ShipGrid.cells` and no block query can retrieve it

### Requirement: Block entities carry only identity and runtime state

A spawned block entity SHALL carry `BlockRef { id }` and `BlockOwner { ship }`; block type, position, and rotation SHALL be read from `ShipGrid` via `BlockRef`. `Block`, `BlockData`, `BlockState`, and `Blocks` SHALL NOT exist.

#### Scenario: Placement is read from the grid
- **WHEN** a system needs a block's type, position, or rotation
- **THEN** it resolves the `BlockRef` in `ShipGrid` and the entity has no placement component

#### Scenario: Block identity components are unique
- **WHEN** a block entity is spawned
- **THEN** it has exactly one `BlockRef` and one `BlockOwner`

### Requirement: All placement changes go through ShipMutation

The system SHALL expose `ShipMutation::{DestroyBlock, DestroyBlocks, PlaceBlock, RebuildFromLayout}` and apply mutations once per fixed tick through `apply_ship_mutations`; no system SHALL mutate `ShipGrid` directly.

#### Scenario: Destruction pipeline
- **WHEN** a block is destroyed
- **THEN** the mutation removes every occupied cell, recomputes connected components, removes core-less fragments, updates stats and bounds, rebuilds the collider, and emits `ShipChanged`

#### Scenario: Placement rejects overlap
- **WHEN** a block is placed on an occupied cell
- **THEN** the mutation rejects it and leaves the grid unchanged

### Requirement: Core-less fragments are removed from grid and collider

After a mutation, the system SHALL compute connected components from `ShipGrid.cells`, find the component containing the core, remove every block in a core-less fragment from the grid, mark those block entities `Death`, and rebuild the collider from live cells only.

#### Scenario: Detached section becomes a fragment
- **WHEN** a block connecting a side section to the core is destroyed
- **THEN** the detached section is removed from the grid, its entities are marked `Death`, and the collider no longer contains its cells

#### Scenario: Core destruction ends the ship
- **WHEN** the core is destroyed
- **THEN** the ship is marked `Death`, no core-less component survives as a ship, and collider rebuild is skipped

### Requirement: Ship identity and stats derive from the core and live grid

`Ship.id` SHALL equal the core block id and `Ship.core_position` SHALL be the core block center in ship-local space. `ShipStats` SHALL be the single stats component and SHALL include mass, max/min range, health/max_health, power aggregates, cargo aggregates, thrust power, and rotate speed. `BoundingBox` SHALL initialize from `i32::MAX`/`i32::MIN` and SHALL be recomputed on every mutation.

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

### Requirement: ShipStats replaces overlapping stat types

The system SHALL use `ShipStats` as the only ship-level stat component; `ComputedShipStats`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipHealth`, `ShipPower`, and a separate `ShipCargo` stat split SHALL NOT exist.

#### Scenario: HUD and AI read one health value
- **WHEN** the HUD or AI needs ship HP
- **THEN** it reads `ShipStats.health`

#### Scenario: Aggregates update together
- **WHEN** storage, power, or thruster blocks change
- **THEN** the corresponding `ShipStats` fields update through the same grid recomputation

### Requirement: Spawn and editor validate ship layouts

Both spawn and editor paths SHALL build the grid through `ShipGrid::rebuild_from_layout` and SHALL validate exactly one core, no overlapping cells, all cells within block shapes, no disconnected non-core components at spawn, no empty turret barrels, and valid power/storage values.

#### Scenario: Overlapping cell is reported
- **WHEN** a design places two blocks in the same cell
- **THEN** validation returns an error naming the cell

#### Scenario: Disconnected hull is rejected
- **WHEN** a design contains a disconnected hull section at spawn
- **THEN** validation rejects the spawn

### Requirement: Ship grid invariants have automated tests

Unit tests SHALL cover multi-cell insert/remove, overlap rejection, negative coordinates, `rebuild_from_layout`, cell counts versus block shapes, stats from grid, fragment removal, core death, bounds updates, and collider cell sets. An integration test SHALL spawn a ship, destroy a mid-ship block, and assert grid, collider, and bounds.

#### Scenario: Grid test suite runs
- **WHEN** `cargo test` runs
- **THEN** the grid, stats, and mutator suites pass and assert no phantom cells
