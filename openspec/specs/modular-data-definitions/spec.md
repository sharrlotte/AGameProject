# Modular Data Definitions

## Purpose

Enforce modular, single-responsibility file boundaries for all data schemas (configs, save snapshots) and establish a unified `Validate` contract with reusable assertion helpers across the engine.

## Requirements

### Requirement: Single-file encapsulation of data kinds
Every distinct configuration schema and save snapshot entity SHALL be defined in its own dedicated file under 150 lines. The file SHALL encapsulate its struct definitions, serde annotations, `Validate` implementation, and domain conversions without mixing unrelated features.

#### Scenario: Block feature config encapsulation
- **WHEN** a developer inspects or modifies `TurretComponentConfig`
- **THEN** all turret-related schema, validation checks, and conversion code reside exclusively in `core/src/config/block/feature/turret.rs`

#### Scenario: Save snapshot entity encapsulation
- **WHEN** a developer inspects or modifies `ShipSnapshot`
- **THEN** the snapshot schema and its structural validation reside exclusively in `core/src/save/world/snapshot/ship.rs`

### Requirement: Unified Validate contract and assertion helpers
The engine SHALL provide a zero-dependency validation module in `core/src/validate/mod.rs` containing a standard `Validate` trait and composable assertion helpers (`check::positive`, `check::non_negative`, `check::range_ordered`, `check::bounded_int`, `check::finite`).

#### Scenario: Config validates via shared check helper
- **WHEN** a config field must be strictly positive
- **THEN** its `Validate` implementation calls `check::positive(field_name, value)` returning a descriptive error on failure
