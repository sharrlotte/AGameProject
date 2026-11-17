## MODIFIED Requirements

### Requirement: WorldSave embeds every ship

`WorldSave` SHALL contain `ships: Vec<ShipSnapshot>` with each ship's unique instance `id`, `team`, `transform`, `velocity`, `cargo` (`cargo_used` and `cargo_items`), `ai` profile and full `ai_runtime` (`current` behavior with target ship `Uuid`s, `spawn_position`, `rule_cooldowns`, and `timer_secs`), and `blocks` (including `placement`, `health`, `max_health`, `turret` state and barrel `rotation`, `turret_charge`, `battery_charge`, `ammo`, and relative weapon pattern states). `WorldSave` SHALL NOT contain `ship_ids`, and no `ships/*.ron` directory SHALL be written or read.

#### Scenario: Two ships from the same design in one battle save
- **WHEN** a world with two ships spawned from the same `ShipDesign` is saved and loaded
- **THEN** each ship has a distinct instance `Ship.id`, both `ShipSnapshot` values appear inside `battle.ron`, and loading restores both ships and their cross-ship AI/camera/pilot targets without `Uuid` collision

### Requirement: Full snapshot fidelity

The snapshot SHALL include battle mission name, elapsed battle seconds, outcome, camera position/zoom/follow target, controlled ship and control mode, ships (with cargo, power charges, turret charges/barrel angles, AI runtime, and relative weapon timers), drops, and RNG state. It SHALL NOT include in-flight bullets, missiles, lasers, beams, damage text, or transient VFX, and those entities SHALL be cleared on load.

#### Scenario: Transforms and velocities round-trip
- **WHEN** two ships at non-zero transforms and velocities are saved and loaded
- **THEN** their transforms and velocities match the snapshot

#### Scenario: Ship cargo, battery charge, and turret state round-trip
- **WHEN** a ship with collected cargo items, partially discharged batteries, partial turret charge, non-zero barrel rotation, and active relative weapon cooldowns is saved and loaded
- **THEN** its `cargo_used`, `cargo_items`, battery `Charge`, `TurretCharge`, barrel rotation, and weapon cooldowns match the snapshot and turrets can fire normally in a fresh session

#### Scenario: AI runtime and pilot/camera targets round-trip
- **WHEN** a battle is saved while the player is piloting a ship and an AI ship is executing a behavior targeting another ship with active rule cooldowns
- **THEN** loading restores `ControlledShip`, `CameraTarget`, `ControlMode`, and the AI ship's exact `AiBehavior` target `Entity`, `spawn_position`, `rule_cooldowns`, and think timer

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

Every save and design document SHALL carry `version`. `WorldSave` SHALL use `SAVE_SCHEMA_VERSION = 4`. The loader SHALL reject a version other than the current schema version with an error naming expected and found versions. Writes SHALL go to `.tmp`, preserve the previous file as `.bak`, and rename only after a successful write; on error the previous save SHALL remain intact.

#### Scenario: Version mismatch fails cleanly
- **WHEN** an incompatible version file (such as v3) is opened
- **THEN** loading returns an incompatibility error and does not spawn a partial world

#### Scenario: Failed write preserves the previous save
- **WHEN** writing fails part-way through
- **THEN** the previous `battle.ron` or `campaign.ron` is still readable

### Requirement: Ship designs are versioned RON documents

The game SHALL store `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, ai: AiProfile }` under `<save_dir>/designs/*.ron`, cache the sorted catalog in a Bevy `Resource` (`ShipCatalog`) seeded once at startup, and update the resource when designs are saved.

#### Scenario: Fresh install has starter designs cached in resource
- **WHEN** the game starts with an empty designs directory
- **THEN** the starter interceptor and raider designs are written, loaded into the `ShipCatalog` resource, and available to the editor and missions without repeated disk seeding

#### Scenario: Missing design is an error
- **WHEN** a mission references a missing design
- **THEN** the mission reports an error and does not fall back to the first ship

#### Scenario: Design round-trip
- **WHEN** a design is saved and reloaded
- **THEN** its blocks and AI profile round-trip exactly and the `ShipCatalog` resource is updated

### Requirement: Save and load round-trips are tested

Tests SHALL cover embedded ship round-trip (including cargo, battery/turret charge, barrel rotation, AI runtime targets, and relative weapon timers), unique ship instance IDs across identical designs, version mismatch (rejecting v3), atomic write behavior, single-file `battle.ron` and `campaign.ron` persistence, design round-trip, starter validation, and absence of per-ship file access.

#### Scenario: Two-ship world round-trip
- **WHEN** the save suite runs
- **THEN** it proves a two-ship world restores entirely from `battle.ron` with full state fidelity
