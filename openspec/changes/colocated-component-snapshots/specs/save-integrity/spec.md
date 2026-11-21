## MODIFIED Requirements

### Requirement: Persisted data is validated at the load boundary
Every deserialization path for `ShipDesign`, `ShipSnapshot`, `DebrisSnapshot`, and `BlockSnapshot` — including both save-slot loading and chunk streaming reload — SHALL call `Validate::validate()` immediately after decode and before any entity is spawned. Validation SHALL reject structural corruption: unknown block types, nil or duplicate block/ship ids, overlapping occupied cells, zero or multiple cores, disconnected layouts, invalid team ids (`>= 16`), out-of-range skill (`!(1..=10)`), non-finite transform/velocity floats, multiple shoot patterns on one block, and dangling ship `Uuid` references. `ShipSnapshot::restore` SHALL NOT re-run `BlockRegistry::validate_ship` after load-boundary validation has already succeeded.

#### Scenario: Corrupt design RON is rejected
- **WHEN** a design file in the sandbox directory contains two blocks on the same cell
- **THEN** the design is skipped with a warning naming the file and the overlapping cell, and no ship is spawned from it

#### Scenario: Invalid team id in a save is rejected
- **WHEN** a saved ship carries a team id of 16 or greater
- **THEN** loading fails with a typed `InvalidData` error naming the ship instead of constructing the team

#### Scenario: Corrupt chunk snapshot is rejected before spawning
- **WHEN** a chunk loaded by `stream_chunks` contains a `ShipSnapshot` with an unknown block type or non-finite transform
- **THEN** validation rejects the snapshot, emits `ChunkStreamError`, and spawns no corrupt entity

### Requirement: Safe-to-clamp values are normalized with a warning
Fields whose valid range is bounded but whose corruption is non-structural (`health`, `max_health`, `battery_charge`, `turret.charge`, `shield.barrier_hp`, `cooldown_remaining`, `remaining_secs`, `cargo.used`, and `Brain` timers) SHALL be clamped or derived inside their owning `Snapshot::from_save` implementation with a warning log. `Validate::validate()` SHALL NOT reject these clampable non-structural fields as fatal load errors.

#### Scenario: Over-capacity cargo is reconciled
- **WHEN** a saved ship reports `used = 0` with a non-empty item manifest
- **THEN** the load succeeds, `used` is derived from the manifest and capacity, and a warning names the ship

#### Scenario: Negative battery charge or remaining lifetime clamps with warning
- **WHEN** a saved block has negative `battery_charge` or a saved debris body has negative `remaining_secs`
- **THEN** load validation succeeds, `from_save` clamps the value to `0.0`, and a warning naming the entity is logged
