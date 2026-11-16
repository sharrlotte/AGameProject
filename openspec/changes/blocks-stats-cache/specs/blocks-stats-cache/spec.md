## ADDED Requirements

### Requirement: A per-ship cell grid is the source of truth for block occupancy

The cache SHALL answer `block_at(cell)` or iterate occupied cells in O(cells
in ship) without rebuilding from `BlockData` lists on every event.

#### Scenario: Collision rebuild uses cached grid
- **WHEN** a block is destroyed and the collider is rebuilt
- **THEN** `cell_map` lookups hit the per-ship grid and no full `HashMap`
  rebuild is performed for the unchanged cells

### Requirement: Cache is maintained incrementally on spawn and despawn

Spawn SHALL insert cell entries; destroy SHALL remove them; no manual
“rebuild now” step SHALL exist for normal play.

#### Scenario: Block destroyed triggers update
- **WHEN** a block with 1×1 size and position (2, 3) is destroyed
- **THEN** the cell `(2, 3)` is no longer reported occupied

### Requirement: Fallback to full rebuild is available and documented

A full grid rebuild SHALL be available (e.g. after editor spawn of a ship
from schematic) and is the correctness fallback, never the default hot path.

#### Scenario: Fresh ship spawns with 80 blocks
- **WHEN** a ship is spawned from a schematic
- **THEN** the grid is populated once and subsequent events read from it
