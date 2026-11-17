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
The save system SHALL store at most one active battle snapshot at `<data_dir>/saves/battle.ron` and one campaign progress document at `<data_dir>/saves/campaign.ron`. Each file SHALL be self-contained.

#### Scenario: Battle save is self-contained
- **WHEN** a battle is saved
- **THEN** only `battle.ron` is written for the battle snapshot and no `slot_*.ron` or per-ship file is created

#### Scenario: Battle load is self-contained
- **WHEN** `battle.ron` is loaded
- **THEN** no `slot_*.ron` or per-ship file is opened

### Requirement: WorldSave embeds every ship

`WorldSave` SHALL contain `ships: Vec<ShipSnapshot>` with each ship's id, team, transform, velocity, AI profile and runtime, and blocks. `WorldSave` SHALL NOT contain `ship_ids`, and no `ships/*.ron` directory SHALL be written or read.

#### Scenario: Two ships in one slot
- **WHEN** a world with two ships is saved
- **THEN** both `ShipSnapshot` values appear inside the slot file and loading restores both from that array

### Requirement: Full snapshot fidelity

The snapshot SHALL include battle mission name, elapsed battle seconds, outcome, camera, ships, drops, and RNG state. It SHALL NOT include in-flight bullets, missiles, lasers, beams, damage text, or transient VFX, and those entities SHALL be cleared on load.

#### Scenario: Transforms and velocities round-trip
- **WHEN** two ships at non-zero transforms and velocities are saved and loaded
- **THEN** their transforms and velocities match the snapshot

#### Scenario: Transients are absent after load
- **WHEN** a bullet, missile, laser, or beam is in flight at save time
- **THEN** no such entity exists after load

#### Scenario: Battle state is restored
- **WHEN** a battle outcome is saved
- **THEN** loading restores `BattlePhase::Ended` and shows the outcome overlay

#### Scenario: Drops are restored
- **WHEN** a drop exists at save time
- **THEN** its position, item type, amount, and remaining lifetime are restored

### Requirement: Saves are versioned and atomic

Every save and design document SHALL carry `version`. The loader SHALL reject a version other than the current `SAVE_SCHEMA_VERSION` with an error naming expected and found versions. Writes SHALL go to `.tmp`, preserve the previous file as `.bak`, and rename only after a successful write; on error the previous save SHALL remain intact.

#### Scenario: Version mismatch fails cleanly
- **WHEN** an incompatible version file is opened
- **THEN** loading returns an incompatibility error and does not spawn a partial world

#### Scenario: Failed write preserves the previous save
- **WHEN** writing fails part-way through
- **THEN** the previous `battle.ron` or `campaign.ron` is still readable

### Requirement: Ship designs are versioned RON documents

The game SHALL store `ShipDesign { version, id, name, blocks: Vec<BlockPlacement>, ai: AiProfile }` under `<save_dir>/designs/*.ron`, provide a sorted catalog, and copy bundled starter designs on first run when the directory is empty.

#### Scenario: Fresh install has starter designs
- **WHEN** the designs directory is empty
- **THEN** the starter interceptor and raider designs are copied and loadable

#### Scenario: Missing design is an error
- **WHEN** a mission references a missing design
- **THEN** the mission reports an error and does not fall back to the first ship

#### Scenario: Design round-trip
- **WHEN** a design is saved and reloaded
- **THEN** its blocks and AI profile round-trip exactly

### Requirement: Hard cutover with no fabrication

The loader SHALL NOT import JSON schematics, read v1 world saves, fabricate phantom ships, or "repair" ships with two or fewer blocks. Old files SHALL remain on disk untouched.

#### Scenario: Legacy-only installation
- **WHEN** only legacy files exist
- **THEN** the UI shows no loadable slots and offers New Ship or New Battle instead of inventing ships

### Requirement: Save and load round-trips are tested

Tests SHALL cover embedded ship round-trip, version mismatch, atomic write behavior, single-file `battle.ron` and `campaign.ron` persistence, design round-trip, starter validation, and absence of per-ship file access.

#### Scenario: Two-ship world round-trip
- **WHEN** the save suite runs
- **THEN** it proves a two-ship world restores entirely from `battle.ron`
