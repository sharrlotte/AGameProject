## MODIFIED Requirements

### Requirement: There is one type per concept

The codebase SHALL use one canonical type per domain concept and SHALL NOT reintroduce removed legacy duplicate types (`BlockData`, `Blocks`, `Block`, `BlockState`, `ShipData`, `ShipState`, `ShipSave`, `BlockSaveData`, `ComputedShipStats`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipFragment`, `StorageBuilder`, `StorageState`, `ShipCargo`, `PowerConsumer`, `PowerStorage`, `SceneEntity`, `Save`, or `BehaviorX` markers).

#### Scenario: Consolidation check
- **WHEN** the codebase is inspected for legacy duplicate types
- **THEN** none of the removed legacy duplicate type names exist

### Requirement: Snapshot capture and restore are trait-driven

World persistence SHALL use direct hierarchical snapshot capture (`BodySnapshot::capture`, `ShipSnapshot::capture`, `DebrisSnapshot::capture`, `BlockSnapshot::capture`) and two-pass restoration (`apply_bodies` resolving cross-ship `Uuid` references via `ship_map`), without dummy-parameter `CaptureSnapshot` / `RestoreSnapshot` traits. `BodySnapshot` SHALL cover both ships and debris in one unified body list.

#### Scenario: Body kinds share one snapshot
- **WHEN** a ship or a debris body is captured
- **THEN** both produce a `BodySnapshot` variant in one unified `Vec<BodySnapshot>`
