## ADDED Requirements

### Requirement: Persisted data is validated at the load boundary
Every deserialization path for `ShipDesign`, `ShipSnapshot`, `DebrisSnapshot`, and `BlockSnapshot` SHALL call a typed `validate()` immediately after decode and before any entity is spawned. Validation SHALL reject unknown block types, duplicate block ids, overlapping occupied cells, zero or multiple cores, invalid team ids, multiple shoot patterns on one block, and references that cannot resolve to a spawned ship.

#### Scenario: Corrupt design RON is rejected
- **WHEN** a design file in the sandbox directory contains two blocks on the same cell
- **THEN** the design is skipped with a warning naming the file and the overlapping cell, and no ship is spawned from it

#### Scenario: Invalid team id in a save is rejected
- **WHEN** a saved ship carries a team id of 16 or greater
- **THEN** loading fails with a typed `InvalidData` error naming the ship instead of constructing the team

### Requirement: Structural corruption rejects the entire load
When validation fails for structural data, the system SHALL abort the load atomically, leave the previously active world untouched, and show the player a notice containing the failing entity and reason. The system SHALL NOT partially restore a world or spawn a replacement save.

#### Scenario: One bad ship aborts the load
- **WHEN** a slot contains one ship with overlapping blocks and otherwise valid data
- **THEN** no entity from that slot is spawned, the notice names the offending ship, and the game remains in its current scene

#### Scenario: Continue failure leaves the slot file untouched
- **WHEN** loading a slot fails validation
- **THEN** the `.db` file is not modified and a later attempt can still target that slot

### Requirement: Safe-to-clamp values are normalized with a warning
Fields whose valid range is bounded but whose corruption is non-structural (weapon charge, cooldown, aim point, cargo totals, remaining lifetime) SHALL be clamped or derived from their source and SHALL emit a warning. They SHALL NOT cause a load rejection.

#### Scenario: Over-capacity cargo is reconciled
- **WHEN** a saved ship reports `used = 0` with a non-empty item manifest
- **THEN** the load succeeds, `used` is derived from the manifest and capacity, and a warning names the ship

### Requirement: The world seed has one authoritative field
`WorldMeta.seed` SHALL be the only persisted seed. `WorldMeta.rng_blob` SHALL NOT exist. Slot listing, world loading, and `GameRng` initialization SHALL all read the same `meta.seed` value. World schema version SHALL be 2.

#### Scenario: Slot picker seed matches the running RNG
- **WHEN** a slot is listed and then loaded
- **THEN** the seed shown in the picker equals the seed used to initialize `GameRng`
