# World Save

## Purpose

RON-only, versioned, atomic single-file battle and campaign saves embed the full world snapshot in one self-contained file.
## Requirements
### Requirement: RON is the only serialization format

The game SHALL read and write world saves and ship designs as RON; `serde_json` and every `.json` code path SHALL be removed from the `core` crate.

#### Scenario: Save and design output is RON
- **WHEN** a world save or ship design is written
- **THEN** the file is RON and no JSON file is produced

#### Scenario: Legacy JSON is ignored
- **WHEN** old JSON schematics or saves exist on disk
- **THEN** the game does not read, rewrite, or delete them

### Requirement: Single battle save and campaign progress files
The save system SHALL store each playthrough as a single self-contained SQLite save slot at `<data_dir>/saves/<slot_id>.db`. No `battle.ron`, `campaign.ron`, global `fleet.ron`, `slot_*.ron`, or per-ship `.ron` save files SHALL be created for save slots. Global user preferences SHALL remain in `<data_dir>/settings.ron`.

#### Scenario: Save slot is a single self-contained SQLite file
- **WHEN** an open-world save slot is created or saved
- **THEN** all world metadata, active entities, and per-slot designs are written into `<data_dir>/saves/<slot_id>.db` and no `battle.ron` or `campaign.ron` is created

#### Scenario: Loading a save slot reads only its database file
- **WHEN** a save slot is loaded from the Save Slot Picker
- **THEN** only that slot's `<slot_id>.db` file is opened

### Requirement: Full snapshot fidelity

The snapshot SHALL include battle mission name, elapsed battle seconds, outcome, camera position/zoom/follow target, controlled ship and control mode, ships (with cargo, power charges, turret charges/barrel angles, `doctrine`, `skill`, `BrainSave` AI runtime, and relative weapon timers), team intel alerts/memory, debris bodies (with blocks, transform, velocity, and remaining lifetime), drops, and RNG state. It SHALL NOT include in-flight bullets, missiles, lasers, beams, damage text, or transient VFX, and those entities SHALL be cleared on load.

#### Scenario: Transforms and velocities round-trip
- **WHEN** two ships at non-zero transforms and velocities are saved and loaded
- **THEN** their transforms and velocities match the snapshot

#### Scenario: Ship cargo, battery charge, and turret state round-trip
- **WHEN** a ship with collected cargo items, partially discharged batteries, partial turret charge, non-zero barrel rotation, and active relative weapon cooldowns is saved and loaded
- **THEN** its `cargo_used`, `cargo_items`, battery `Charge`, `TurretCharge`, barrel rotation, and weapon cooldowns match the snapshot and turrets can fire normally in a fresh session

#### Scenario: AI runtime and pilot/camera targets round-trip
- **WHEN** a battle is saved while the player is piloting a ship and an AI ship is executing a `Tactic` targeting another ship with a `Doctrine`, `Skill`, and `shed_boosters` state
- **THEN** loading restores `ControlledShip`, `CameraTarget`, `ControlMode`, and the AI ship's `Doctrine`, `Skill`, `Tactic` target `Entity`, `spawn_position`, `think_timer`, and `shed_boosters` state

#### Scenario: Debris block state round-trips
- **WHEN** a debris body with damaged blocks and remaining lifetime is saved and loaded
- **THEN** each block's placement and health, the body's team, transform, velocity, and remaining lifetime match the snapshot

#### Scenario: Transients are absent after load
- **WHEN** a bullet, missile, laser, or beam is in flight at save time
- **THEN** no such entity exists after load

#### Scenario: Battle state is restored
- **WHEN** a battle outcome is saved
- **THEN** loading restores `BattlePhase::Ended` and shows the outcome overlay

#### Scenario: Drops are restored with visible sprites
- **WHEN** a drop exists at save time
- **THEN** its position, item type, amount, remaining lifetime, and visible `Sprite` (colored by rarity and sized by `drop_size`) are restored

### Requirement: Saves are versioned and atomic
Every save slot database SHALL carry `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION`. The loader SHALL reject a version other than the current schema version with an error naming expected and found versions. All save writes SHALL commit inside an ACID SQLite transaction so a failure leaves the previous committed state intact.

#### Scenario: Version mismatch fails cleanly
- **WHEN** an incompatible `.db` version is opened
- **THEN** loading returns an incompatibility error and does not spawn a partial world

#### Scenario: Failed transaction preserves previous committed save
- **WHEN** a write transaction fails or rolls back
- **THEN** the previously committed state in `<slot_id>.db` remains intact and readable

### Requirement: Ship designs are versioned RON documents

The game SHALL store `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, doctrine: Doctrine }` with `DESIGN_SCHEMA_VERSION = 3` under `<save_dir>/designs/*.ron`, where `BlockPlacement.rotation` is a `Rot90` serialized by variant name (`R0`, `R90`, `R180`, `R270`). The game SHALL cache the sorted catalog in a Bevy `Resource` (`ShipCatalog`) seeded once at startup, and update the resource when designs are saved.

#### Scenario: Fresh install has starter designs cached in resource
- **WHEN** the game starts with an empty designs directory
- **THEN** the v3 starter interceptor, raider, B2 and Blackbird designs are written, loaded into the `ShipCatalog` resource, and available to the editor and missions without repeated disk seeding

#### Scenario: Missing design is an error
- **WHEN** a mission references a missing design
- **THEN** the mission reports an error and does not fall back to the first ship

#### Scenario: Design round-trip
- **WHEN** a design containing a rotated block and explicit `Doctrine` is saved and reloaded
- **THEN** its blocks, including `rotation`, and `doctrine` round-trip exactly and the `ShipCatalog` resource is updated

#### Scenario: v2 design rejected
- **WHEN** a v2 design file is read
- **THEN** it is reported as incompatible, left untouched on disk, and not loaded

### Requirement: Hard cutover with no fabrication
The loader SHALL NOT import legacy JSON schematics, v1–v7 `battle.ron`, `campaign.ron`, or `fleet.ron`, fabricate phantom ships, or repair invalid ships. Legacy files SHALL remain on disk untouched.

#### Scenario: Legacy-only installation
- **WHEN** only legacy `.ron` or `.json` save files exist on disk
- **THEN** the Save Slot Picker shows no loadable slots and offers `New World` without deleting or migrating legacy files

### Requirement: Save and load round-trips are tested

Tests SHALL cover embedded ship round-trip (including cargo, battery/turret charge, barrel rotation, block placement rotation, `Doctrine`, `Skill`, `BrainSave` targets, and relative weapon timers), embedded debris round-trip with remaining lifetime, unique ship instance IDs across identical designs, version mismatch (rejecting v6 world saves and v2 designs), atomic write behavior, single-file `battle.ron` and `campaign.ron` persistence, design round-trip, starter validation, and absence of per-ship file access.

#### Scenario: Two-ship world round-trip
- **WHEN** the save suite runs
- **THEN** it proves a two-ship world restores entirely from `battle.ron` with full state fidelity

#### Scenario: Mixed world round-trip
- **WHEN** the save suite saves one ship and one debris body
- **THEN** loading restores both bodies and the debris never resolves as a camera, control, or AI target

### Requirement: WorldSave embeds every body

`WorldSave` SHALL contain `bodies: Vec<BodySnapshot>` where `BodySnapshot` is either `Ship(ShipSnapshot)` or `Debris(DebrisSnapshot)`. A ship snapshot SHALL carry its unique instance `id`, `team`, `transform`, `velocity`, `cargo`, `doctrine`, `skill`, full `brain` runtime (`BrainSave`), and `blocks`. A debris snapshot SHALL carry `team`, `transform`, `velocity`, `remaining_secs`, and `blocks`; debris SHALL NOT require an `id` because no camera, control, AI, or mission reference targets it. `WorldSave` SHALL NOT contain `ship_ids`, and no `ships/*.ron` directory SHALL be written or read.

#### Scenario: Two ships from the same design in one battle save
- **WHEN** a world with two ships spawned from the same `ShipDesign` is saved and loaded
- **THEN** each ship has a distinct instance `Ship.id`, both `BodySnapshot::Ship` values appear inside `battle.ron`, and loading restores both ships and their cross-ship AI/camera/pilot targets without `Uuid` collision

#### Scenario: Debris is embedded with its lifetime
- **WHEN** a world with a debris body is saved and loaded
- **THEN** the debris body appears as `BodySnapshot::Debris`, its blocks and transform round-trip, and it keeps its remaining lifetime

