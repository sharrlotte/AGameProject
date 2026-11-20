## MODIFIED Requirements

### Requirement: Persisted data is validated at the load boundary
Every deserialization path for `ShipDesign`, `ShipSnapshot`, `DebrisSnapshot`, and `BlockSnapshot` SHALL call `Validate::validate()` defined in its respective dedicated snapshot module immediately after decode and before any entity is spawned. Validation SHALL reject unknown block types, duplicate block ids, overlapping occupied cells, zero or multiple cores, invalid team ids, multiple shoot patterns on one block, and references that cannot resolve to a spawned ship. Non-structural transient values SHALL be normalized separately in `sanitize.rs`.

#### Scenario: Corrupt design RON is rejected
- **WHEN** a design file in the sandbox directory contains two blocks on the same cell
- **THEN** the design is skipped with a warning naming the file and the overlapping cell, and no ship is spawned from it

#### Scenario: Invalid team id in a save is rejected
- **WHEN** a saved ship carries a team id of 16 or greater
- **THEN** loading fails with a typed `InvalidData` error naming the ship instead of constructing the team
