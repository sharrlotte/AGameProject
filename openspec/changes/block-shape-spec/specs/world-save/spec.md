## MODIFIED Requirements

### Requirement: Saves are versioned and atomic

Every save and design document SHALL carry `version`. `WorldSave` SHALL use `SAVE_SCHEMA_VERSION = 5`. The loader SHALL reject a version other than the current schema version with an error naming expected and found versions. Writes SHALL go to `.tmp`, preserve the previous file as `.bak`, and rename only after a successful write; on error the previous save SHALL remain intact.

#### Scenario: Version mismatch fails cleanly
- **WHEN** an incompatible version file (such as v4) is opened
- **THEN** loading returns an incompatibility error and does not spawn a partial world

#### Scenario: Failed write preserves the previous save
- **WHEN** writing fails part-way through
- **THEN** the previous `battle.ron` or `campaign.ron` is still readable

### Requirement: Ship designs are versioned RON documents

The game SHALL store `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, ai: AiProfile }` with `DESIGN_SCHEMA_VERSION = 2` under `<save_dir>/designs/*.ron`, where `BlockPlacement.rotation` is a `Rot90` serialized by variant name (`R0`, `R90`, `R180`, `R270`). The game SHALL cache the sorted catalog in a Bevy `Resource` (`ShipCatalog`) seeded once at startup, and update the resource when designs are saved.

#### Scenario: Fresh install has starter designs cached in resource
- **WHEN** the game starts with an empty designs directory
- **THEN** the v2 starter interceptor and raider designs are written, loaded into the `ShipCatalog` resource, and available to the editor and missions without repeated disk seeding

#### Scenario: Missing design is an error
- **WHEN** a mission references a missing design
- **THEN** the mission reports an error and does not fall back to the first ship

#### Scenario: Design round-trip
- **WHEN** a design containing a rotated block is saved and reloaded
- **THEN** its blocks, including `rotation`, and AI profile round-trip exactly and the `ShipCatalog` resource is updated

#### Scenario: v1 design rejected
- **WHEN** a v1 design file is read
- **THEN** it is reported as incompatible, left untouched on disk, and not loaded

### Requirement: Save and load round-trips are tested

Tests SHALL cover embedded ship round-trip (including cargo, battery/turret charge, barrel rotation, block placement rotation, AI runtime targets, and relative weapon timers), unique ship instance IDs across identical designs, version mismatch (rejecting v4 world saves and v1 designs), atomic write behavior, single-file `battle.ron` and `campaign.ron` persistence, design round-trip, starter validation, and absence of per-ship file access.

#### Scenario: Two-ship world round-trip
- **WHEN** the save suite runs
- **THEN** it proves a two-ship world restores entirely from `battle.ron` with full state fidelity
