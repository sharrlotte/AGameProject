# Code Foundation

## Purpose

String keys, centralized RON document IO, snapshot and feature macros, and one type per concept form the shared code foundation.

## Requirements
### Requirement: String keys replace interned symbols

Block, item, and type keys SHALL be `String` values; `Symbol`, `Symbols`, `src/symbol.rs`, and the `lasso` dependency SHALL NOT exist.

#### Scenario: Registry lookup without an interner
- **WHEN** a block is looked up
- **THEN** `BlockRegistry::get(&str)` returns its definition with no interner

#### Scenario: Removed dependencies
- **WHEN** `Cargo.toml` is inspected
- **THEN** neither `lasso` nor `serde_json` is present

### Requirement: There is one type per concept

The codebase SHALL use the canonical types from the design consolidation table and SHALL NOT contain `BlockData`, `Blocks`, `Block`, `BlockState`, `ShipData`, `ShipState`, `ShipSave`, `BlockSaveData`, `TurretSaveState`, `ComputedShipStats`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipFragment`, `Storage`, `StorageBuilder`, `StorageState`, `ShipCargo`, `Ammo`, `AmmoBuilder`, `PowerConsumer`, `PowerStorage`, `SceneEntity`, `Save`, or the `BehaviorX` markers.

#### Scenario: Consolidation check
- **WHEN** the type-consolidation test greps the source
- **THEN** only canonical types remain

### Requirement: Config and runtime splits only exist where fields differ materially

`BlockConfig` and `BlockDef`, `FeatureConfig` and `Feature`, `ShipDesign` and `Ship`, and `AiProfile` and `AiRuntime` MAY stay split because they carry asset paths versus handles or static versus live state; all other DTO pairs SHALL be merged.

#### Scenario: New feature does not duplicate structs
- **WHEN** a new feature is added
- **THEN** it does not create a duplicate config/runtime struct without differing fields

### Requirement: RON document IO is centralized

The system SHALL provide `RonDocument`, `read_document`, `write_document`, and `atomic_write`, and SHALL use them for world saves and ship designs. Per-type `fs::read_to_string`, `ron::from_str`, and duplicate version checks SHALL NOT exist.

#### Scenario: One document path
- **WHEN** a design is read or written
- **THEN** the same document path performs the version check and atomic write

### Requirement: Snapshot capture and restore are trait-driven

`CaptureSnapshot` and `RestoreSnapshot` SHALL be implemented for `WorldSave`, `ShipSnapshot`, `BlockSnapshot`, and `ItemDrop`, and `save_world` and `load_world` SHALL only orchestrate them.

#### Scenario: New snapshot field
- **WHEN** a new snapshot field is added
- **THEN** its capture and restore live in the corresponding impl rather than a copy-pasted loop

### Requirement: Feature mapping is macro-generated

`feature_set!` SHALL generate the config and runtime enums, conversion, insertion, and validation so adding a feature is one declaration.

#### Scenario: One-line feature addition
- **WHEN** a new feature variant is declared
- **THEN** conversion, insertion, and validation compile without edits in five files

### Requirement: Shared stat macros remove duplicated pipelines

`make_ship_stat!` and a generated registration helper SHALL extend the existing stat macro family; power and efficiency stats SHALL use the same pipeline without duplicate query loops.

#### Scenario: New stat uses the macro
- **WHEN** a new ship stat is added
- **THEN** it uses the macro and does not require a plugin edit

### Requirement: RON block config stays keyed and embedded enums stay data

`block.ron` SHALL remain a mapping of id to definition with duplicate ids rejected, and `ShootingProjectile`, `TargetingStrategy`, and `ShootPattern` SHALL remain plain serde enums.

#### Scenario: Duplicate id rejected
- **WHEN** `block.ron` declares a duplicate id
- **THEN** boot reports a duplicate-id error

#### Scenario: Turret uses its embedded enum
- **WHEN** a turret fires
- **THEN** its projectile and pattern enums are used directly with no registry round-trip

### Requirement: Tests exist from the baseline onward

Each phase SHALL add unit or integration tests, and `cargo test` SHALL be part of the acceptance gate.

#### Scenario: Phase completion
- **WHEN** a phase is marked done
- **THEN** its tests run green and cover the phase invariants
