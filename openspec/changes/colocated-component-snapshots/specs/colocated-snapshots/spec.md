## ADDED Requirements

### Requirement: Persistable gameplay components co-locate Snapshot conversions in their owning modules
The engine SHALL provide a typed `Snapshot` trait (`to_save` / `from_save` with explicit `Save`, `CaptureCtx`, and `RestoreCtx` associated types) in a leaf module with no dependency on `save/`. Each persistable component (`Health`, `Charge`, `TurretState`, `TurretCharge`, `AmmoState`, weapon pattern states, `Shield`, `Brain`, `Tactic`, `TeamIntel`, ship cargo, `Transform`, `Velocity`, `Lifetime`) SHALL define its save DTO and `impl Snapshot` in its owning module rather than duplicating field mappings inside `save/world/snapshot/capture.rs` and `apply.rs`.

#### Scenario: Owning module encapsulates conversion and sanitization
- **WHEN** a field is added to `Brain` or `Shield` in its owning module
- **THEN** its `Save` struct, `to_save`, and `from_save` live in that same module and compile-time exhaustive destructuring enforces updating both directions

#### Scenario: Aggregate snapshots delegate without dummy parameters
- **WHEN** `BlockSnapshot`, `ShipSnapshot`, or `DebrisSnapshot` captures or restores an entity
- **THEN** it assembles owning-module `Snapshot` conversions directly with typed contexts and zero `Entity::PLACEHOLDER` or `DynamicScene` reflection indirection

### Requirement: Shield state, booster shedding, tracking state, and AI timers round-trip with full fidelity
`BlockSnapshot` SHALL persist `Shield` state (`Active`, `Depleted { cooldown_remaining }`, `Regenerating`) plus barrier HP, explicit block `Disabled` state (booster shedding on live ships), full `TurretState::{Idle, Tracking, Ready}`, and `Brain` timer duration + remaining seconds.

#### Scenario: Depleted shield and shed booster round-trip across save and load
- **WHEN** a ship with a depleted shield (3.5s cooldown remaining) and a shed (`Disabled`) booster is saved and reloaded
- **THEN** the restored shield resumes in `Depleted { cooldown_remaining: 3.5 }` and the restored booster retains `Disabled`

#### Scenario: Turret in Tracking state preserves aim state
- **WHEN** a turret in `TurretState::Tracking` is saved and reloaded
- **THEN** it restores as `TurretState::Tracking` with its saved aim point rather than collapsing to `Idle`
