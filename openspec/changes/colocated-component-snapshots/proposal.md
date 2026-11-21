## Why

Adding or changing any persisted field currently requires editing 4–6 separate artifacts across `core/src/`: the runtime component (`content/...`), snapshot DTOs (`save/world/snapshot/{block,ship,debris,world}.rs`), capture logic (`capture.rs`), restore logic (`apply.rs`), validation (`ship.rs`/`block.rs`), and sanitization (`sanitize.rs`). This fan-out has already caused real persistence drift: `Shield` state/HP and booster shedding (`Disabled`) are completely missing from `BlockSnapshot`, `TurretState::Tracking` collapses into `bool`, `Brain.think_timer` saves duration instead of remaining time, `sanitize_remaining_secs` is dead code, `BlockSnapshot::validate` hard-rejects clampable charges before `sanitize.rs` can normalize them, and `registry.validate_ship` runs twice during restore while chunk reload skips validation entirely.

## What Changes

- Define a small, typed `Snapshot` trait (`to_save` / `from_save` with explicit associated `Save` and read/resolve context types, no dummy `Entity::PLACEHOLDER` parameters and no `DynamicScene`/reflection coupling) co-located with each persistable component in its owning module (`Health`, `Charge`, `TurretState`/`TurretCharge`, `AmmoState`, `ShootPattern` states, `Shield`, `Disabled`, `Brain`/`Tactic`, `TeamIntel`, `ShipStats` cargo, `Transform`/`Velocity`).
- Refactor `BlockSnapshot`, `ShipSnapshot`, `DebrisSnapshot`, and `WorldSnapshot` to assemble these owning-module `Snapshot` conversions rather than reaching into private/internal runtime fields from `capture.rs` and `apply.rs`.
- **BREAKING** Bump `WORLD_DB_SCHEMA_VERSION` when adding persisted `ShieldSave` (state, cooldown, barrier HP), `disabled: bool` (booster shedding), `TurretSaveState` (`Idle | Tracking | Ready`), and `BrainSave.think_remaining_secs` to `BlockSnapshot` / `ShipSnapshot` (hard cutover per project policy).
- Reconcile validation vs. sanitization: structural violations reject load via `Validate`, while bounded non-structural values (`charge`, `cooldown`, `remaining_secs`, `cargo.used`) normalize cleanly in owning `from_save` / `sanitize` helpers without contradictory hard-rejection in `BlockSnapshot::validate` / `DebrisSnapshot::validate`.
- Eliminate duplicate `registry.validate_ship` execution during `ShipSnapshot::restore`, and validate decoded `ShipSnapshot`/`DebrisSnapshot` on chunk streaming reload as well.

## Capabilities

### New Capabilities
- `colocated-snapshots`: Typed `Snapshot` conversion trait co-located with owning gameplay components so save DTOs, `to_save`, `from_save`, and field sanitization live next to their runtime definitions while aggregate snapshots remain thin assemblers.

### Modified Capabilities
- `save-integrity`: Enforce consistent structural-reject vs. non-structural-sanitize boundaries (including `Shield`, `Disabled`, `TurretState`, and `remaining_secs`), run validation on chunk reloads as well as slot loads, and eliminate double `validate_ship` execution.

## Non-Goals

- Reviving the removed `CaptureSnapshot` / `RestoreSnapshot` traits with dummy `Entity::PLACEHOLDER` arguments.
- Using Bevy `DynamicScene` or reflection-based serialization (couples save format to ECS internals and complicates schema evolution).
- Changing async I/O or chunk table layout (handled by `async-save-io` and `chunk-storage-cache-and-layout`).

## Impact

- `core/src/save/world/snapshot/` (`mod.rs`, `trait.rs` or `codec.rs`, `block.rs`, `ship.rs`, `debris.rs`, `world.rs`, `capture.rs`, `apply.rs`, `sanitize.rs`).
- Owning gameplay component modules (`content/block/feature/{turret,shield,power,ammo}`, `content/ship/ai/`, `content/ship/stat.rs`, `content/lifetime/`).
- `core/src/world/chunk/system.rs` (validate chunk snapshots before restore).
- `docs/save.md` (spec-only update).
