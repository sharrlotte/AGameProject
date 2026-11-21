## 1. Define Leaf `Snapshot` Trait and Shared Sanitization Helpers

- [ ] 1.1 Define the typed `Snapshot` trait (`Save`, `CaptureCtx`, `RestoreCtx`, `to_save`, `from_save`) in `core/src/validate/snapshot.rs` (re-exported from `core/src/validate/mod.rs`) with zero dependency on `save/`.
- [ ] 1.2 Move reusable numeric clamping/warning helpers (`clamp_health`, `clamp_charge`, `clamp_remaining_secs`) into `core/src/validate/sanitize.rs` so owning modules can use them directly inside `from_save`.

## 2. Co-locate `Snapshot` Impls in Owning Gameplay Modules

- [ ] 2.1 Implement `Snapshot` for `Health`, `Charge`, `AmmoState`, `TurretState`/`TurretCharge` (preserving `Tracking`), and shoot pattern states inside their owning modules under `core/src/content/block/`.
- [ ] 2.2 Implement `Snapshot` for `Shield` (`ShieldSave { state, barrier_hp }`) in `core/src/content/block/feature/shield/` and support restoring initial barrier HP on spawn.
- [ ] 2.3 Implement `Snapshot` for `Tactic`, `Brain` (saving duration + remaining secs), and `TeamIntel` inside `core/src/content/ship/ai/`, and for ship cargo (`CargoSave`) in `core/src/content/ship/stat.rs`.
- [ ] 2.4 Implement `Snapshot` for `Transform` (`TransformSave`), `Velocity` (`VelocitySave`), and `Lifetime` (`remaining_secs`) cleanly without duplicating conversion code between ships and debris.

## 3. Refactor Aggregate Snapshots and Validation Boundaries

- [ ] 3.1 Update `BlockSnapshot` (`core/src/save/world/snapshot/block.rs`) to assemble owning-module `Save` structs, add `shield: Option<ShieldSave>` and `disabled: bool`, and remove hard-rejection of clampable charges.
- [ ] 3.2 Update `ShipSnapshot`, `DebrisSnapshot`, `capture.rs`, and `apply.rs` to delegate to owning `Snapshot` impls, eliminate duplicate `registry.validate_ship` during restore, and add finite-float checks on transforms/velocities.
- [ ] 3.3 Update `world/chunk/system.rs` (`load_in_range`) to validate decoded `ShipSnapshot` and `DebrisSnapshot` entries before calling `apply_bodies`.
- [ ] 3.4 Bump `WORLD_DB_SCHEMA_VERSION` cleanly (or coordinate schema bump with chunk-layout change) and delete dead `save/world/snapshot/sanitize.rs` code replaced by leaf helpers.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests verifying round-trip fidelity for `Shield` (`Active`/`Depleted`/`Regenerating` + barrier HP), `Disabled` booster shedding, `TurretState::Tracking`, and `Brain` remaining timer.
- [ ] 4.2 Add unit tests proving negative charges/lifetimes clamp with warning (no load abort), structural corruption in chunk reloads is rejected, and `validate_ship` runs once per restore.
- [ ] 4.3 Update `docs/save.md` (spec only, no notes) documenting co-located `Snapshot` conversions and full block/shield/disabled persistence.
