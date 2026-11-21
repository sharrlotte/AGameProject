## MODIFIED Requirements

### Requirement: RON is the only serialization format

Global preferences (`setting.ron`), shipped game config files, and Home sandbox ship designs (`<data_dir>/sandbox_designs/*.ron`) SHALL use RON; `serde_json` and every `.json` code path SHALL remain absent from the `core` crate. Save-slot databases SHALL use SQLite with `postcard` + `lz4_flex` binary payloads.

#### Scenario: Sandbox design output is RON
- **WHEN** a Home sandbox ship design is written
- **THEN** the file is RON and no JSON file is produced

#### Scenario: Legacy JSON is ignored
- **WHEN** old JSON schematics or saves exist on disk
- **THEN** the game does not read, rewrite, or delete them

### Requirement: Full snapshot fidelity

The active world snapshot (`WorldMeta` + `ActiveStateSnapshot`) SHALL include world seed, accumulated play time (`SessionClock`), camera position/zoom/follow target, controlled ship and control mode, active ships (with cargo, power charges, turret charges/barrel angles, `doctrine`, `skill`, `BrainSave` AI runtime, and relative weapon timers), team intel alerts/memory, debris bodies (with blocks, transform, velocity, and remaining lifetime), and item drops with visible sprites. It SHALL NOT include in-flight bullets, missiles, lasers, beams, or transient VFX, and those entities SHALL be cleared on load.

#### Scenario: Transforms and velocities round-trip
- **WHEN** two ships at non-zero transforms and velocities are saved and loaded
- **THEN** their transforms and velocities match the snapshot

#### Scenario: Ship cargo, battery charge, and turret state round-trip
- **WHEN** a ship with collected cargo items, partially discharged batteries, partial turret charge, non-zero barrel rotation, and active relative weapon cooldowns is saved and loaded
- **THEN** its `cargo_used`, `cargo_items`, battery `Charge`, `TurretCharge`, barrel rotation, and weapon cooldowns match the snapshot and turrets fire normally in a fresh session

#### Scenario: AI runtime and pilot/camera targets round-trip
- **WHEN** a world is saved while the player is piloting a ship and an AI ship is executing a `Tactic` targeting another ship with a `Doctrine` and `Skill`
- **THEN** loading restores `ControlledShip`, `CameraTarget`, `ControlMode`, and the AI ship's `Doctrine`, `Skill`, `Tactic` target `Entity`, `spawn_position`, and `think_timer`

#### Scenario: Debris block state round-trips
- **WHEN** a debris body with damaged blocks and remaining lifetime is saved and loaded
- **THEN** each block's placement and health, the body's team, transform, velocity, and remaining lifetime match the snapshot

#### Scenario: Transients are absent after load
- **WHEN** a bullet, missile, laser, or beam is in flight at save time
- **THEN** no such entity exists after load

#### Scenario: Drops are restored with visible sprites
- **WHEN** a drop exists at save time
- **THEN** its position, item type, amount, remaining lifetime, and visible `Sprite` (colored by rarity and sized by `drop_size`) are restored

### Requirement: Ship designs are versioned RON documents

The game SHALL store Home sandbox `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, doctrine: Doctrine }` with `DESIGN_SCHEMA_VERSION = 3` under `<data_dir>/sandbox_designs/*.ron`, where `BlockPlacement.rotation` is a `Rot90` serialized by variant name (`R0`, `R90`, `R180`, `R270`). Per-slot designs SHALL be stored in the slot's SQLite `designs` table. The game SHALL cache the sorted catalog in `ShipCatalog` and update it when designs are saved.

#### Scenario: Fresh install has starter designs cached in resource
- **WHEN** the game starts with an empty sandbox designs directory
- **THEN** the v3 starter designs (`interceptor`, `raider`, `b2`, `blackbird`, `aegis`, `vanguard`) are seeded and loaded into `ShipCatalog`

#### Scenario: Design round-trip
- **WHEN** a design containing a rotated block and explicit `Doctrine` is saved and reloaded
- **THEN** its blocks, including `rotation`, and `doctrine` round-trip exactly and `ShipCatalog` is updated

#### Scenario: v2 design rejected
- **WHEN** a v2 design file is read
- **THEN** it is reported as incompatible, left untouched on disk, and not loaded

### Requirement: Save and load round-trips are tested

Tests SHALL cover embedded ship round-trip (including cargo, battery/turret charge, barrel rotation, block placement rotation, `Doctrine`, `Skill`, `BrainSave` targets, and relative weapon timers), embedded debris round-trip with remaining lifetime, unique ship instance IDs across identical designs, schema version mismatch (rejecting incompatible `.db` versions and v2 designs), atomic SQLite transactions, design round-trip, and starter validation.

#### Scenario: Two-ship world round-trip
- **WHEN** the save suite runs
- **THEN** it proves a two-ship world restores from SQLite `active_state` + `world_meta` with full state fidelity

#### Scenario: Mixed world round-trip
- **WHEN** the save suite saves one ship and one debris body
- **THEN** loading restores both bodies and the debris never resolves as a camera, control, or AI target

### Requirement: WorldSave embeds every body

`ActiveStateSnapshot.bodies_blob` SHALL encode `Vec<BodySnapshot>` where `BodySnapshot` is either `Ship(ShipSnapshot)` or `Debris(DebrisSnapshot)`. A ship snapshot SHALL carry its unique instance `id`, `team`, `transform`, `velocity`, `cargo`, `doctrine`, `skill`, full `brain` runtime (`BrainSave`), and `blocks`. A debris snapshot SHALL carry `team`, `transform`, `velocity`, `remaining_secs`, and `blocks`; debris SHALL NOT require an `id`. No per-ship `ships/*.ron` files SHALL be written or read.

#### Scenario: Two ships from the same design in one world save
- **WHEN** a world with two ships spawned from the same `ShipDesign` is saved and loaded
- **THEN** each ship has a distinct instance `Ship.id`, both `BodySnapshot::Ship` values appear in `active_state.bodies_blob`, and loading restores both ships and their cross-ship targets without `Uuid` collision

#### Scenario: Debris is embedded with its lifetime
- **WHEN** a world with a debris body is saved and loaded
- **THEN** the debris body appears as `BodySnapshot::Debris`, its blocks and transform round-trip, and it keeps its remaining lifetime
