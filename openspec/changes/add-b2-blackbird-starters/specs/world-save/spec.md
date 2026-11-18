## MODIFIED Requirements

### Requirement: Ship designs are versioned RON documents

The game SHALL store `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, ai: AiProfile }` with `DESIGN_SCHEMA_VERSION = 2` under `<save_dir>/designs/*.ron`, where `BlockPlacement.rotation` is a `Rot90` serialized by variant name (`R0`, `R90`, `R180`, `R270`). The game SHALL cache the sorted catalog in a Bevy `Resource` (`ShipCatalog`) seeded once at startup, and update the resource when designs are saved.

#### Scenario: Fresh install has starter designs cached in resource

- **WHEN** the game starts with an empty designs directory
- **THEN** the v2 starter interceptor, raider, B2 and Blackbird designs are written, loaded into the `ShipCatalog` resource, and available to the editor and missions without repeated disk seeding

#### Scenario: Missing design is an error

- **WHEN** a mission references a missing design
- **THEN** the mission reports an error and does not fall back to the first ship

#### Scenario: Design round-trip

- **WHEN** a design containing a rotated block is saved and reloaded
- **THEN** its blocks, including `rotation`, and AI profile round-trip exactly and the `ShipCatalog` resource is updated

#### Scenario: v1 design rejected

- **WHEN** a v1 design file is read
- **THEN** it is reported as incompatible, left untouched on disk, and not loaded
