# Ship Cargo

## Purpose

Shared per-ship cargo pools capacity from storage blocks and collects drops atomically with player-only pickup.
## Requirements
### Requirement: Cargo is shared per ship

Each ship SHALL store cargo in `ShipStats` (`cargo_capacity`, `cargo_used`, `cargo_items`); a separate `ShipCargo` component and per-block storage inventories SHALL NOT exist.

#### Scenario: Two storage blocks, one pool
- **WHEN** a ship has two storage blocks
- **THEN** both contribute to one cargo pool and pickup checks the pool once

### Requirement: Capacity is pooled from storage blocks

Cargo capacity SHALL equal the sum of `StorageConfig.capacity` over all storage blocks in `BlockGrid` and SHALL be recomputed on `ShipChanged`. Debris bodies have no cargo capacity.

#### Scenario: Pooled capacity
- **WHEN** storage blocks of 500 and 500 exist
- **THEN** capacity is 1000

#### Scenario: Capacity after destruction
- **WHEN** a storage block is destroyed
- **THEN** capacity drops by its value and overflow handling follows the atomic rule

### Requirement: Pickup is atomic

A drop SHALL be collected only if its full amount fits in cargo; otherwise it SHALL remain in the world unchanged. The drop entity SHALL be despawned only after a successful add.

#### Scenario: Partial fit is rejected
- **WHEN** cargo has 3 free and a drop has 5
- **THEN** the drop is not collected

#### Scenario: Exact fit is collected
- **WHEN** cargo has 5 free and a drop has 5
- **THEN** the drop is collected and `ShipStats.cargo_used` increases by 5

### Requirement: Player ships collect drops

Only player ships SHALL collect drops in this pass; enemy loot SHALL remain world loot. Collection SHALL find the nearest player ship with space, magnet the drop toward it, and pick up within the pickup radius.

#### Scenario: Only the player collects
- **WHEN** a drop is near an enemy ship but a player ship has space
- **THEN** only the player ship collects it

### Requirement: Cargo is visible

The HUD SHALL show `CARGO used / capacity` for the controlled ship, and the pause or editor stats panel SHALL show an item breakdown. Updates SHALL occur on cargo change, not every frame.

#### Scenario: HUD updates on change
- **WHEN** cargo changes
- **THEN** the HUD text updates without a per-frame rebuild

### Requirement: Per-block storage types are deleted

`Storage`, `StorageBuilder`, and `StorageState` SHALL NOT exist; storage blocks SHALL only contribute capacity.

#### Scenario: Storage block has no inventory
- **WHEN** a storage block spawns
- **THEN** it has no storage inventory component

