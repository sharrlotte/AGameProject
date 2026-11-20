## Context

Config files (`.ron`) and world snapshots (binary SQLite BLOBs) currently define their data structures, serde mappings, conversion logic, and validation checks in large centralized files (`core/src/config/block.rs` ~450 lines, `core/src/save/world/snapshot/component.rs` ~420 lines). Validation is invoked through inconsistent ad-hoc functions (`validate(&self) -> Result<(), String>`, `.validated() -> Self`, `validate_projectile`, `validate_blocks`), and configs are only validated after asset loading in secondary Bevy systems (`check_config_ready`). This violates single responsibility, increases incremental compilation times, and allows invalid data states to enter memory before being checked.

## Goals / Non-Goals

**Goals:**
- Provide a zero-dependency, zero-proc-macro validation trait `Validate` with composable assertions in `core/src/validate/mod.rs`.
- Split each kind of data into its own dedicated file (e.g. `turret.rs`, `shield.rs`, `thruster.rs`, `ship.rs`, `block.rs`) containing only its schema, its `Validate` implementation, and its domain conversion.
- Enforce decode-time validation in `RonAssetLoader` so invalid `.ron` files fail immediately at load with line, column, and field details.
- Standardize snapshot integrity checks under `Validate` while isolating runtime value clamping/normalization in `sanitize.rs`.
- Ensure all files remain well under 150 lines, respecting project guidelines.

**Non-Goals:**
- Introducing external validation crates like `garde` or `validator` (rejected to maintain sub-second compile times and avoid heavy proc macros).
- Validating runtime components inside hot simulation tick systems (runtime bounds remain guarded by validated setters on private fields).
- Modifying SQLite table schema or changing the postcard serialization format.

## Decisions

1. **Shared Zero-Dependency Validation Trait (`core/src/validate/mod.rs`):**
   - Define `pub trait Validate { type Context; type Error: std::fmt::Display; fn validate(&self, ctx: &Self::Context) -> Result<(), Self::Error>; }`.
   - Provide assertion helpers under `validate::check`: `positive`, `non_negative`, `range_ordered`, `bounded_int`, and `finite`.
   - *Rationale:* Single source of truth across configs, snapshots, and tool builders with zero compilation overhead.

2. **Single-File-Per-Data-Kind Module Layout:**
   - Deconstruct `core/src/config/block.rs`:
     - `core/src/config/block/mod.rs` (module facade & `BlocksConfig`)
     - `core/src/config/block/def.rs` (`BlockDefConfig`)
     - `core/src/config/block/feature/mod.rs` (`FeatureConfig` enum)
     - `core/src/config/block/feature/turret.rs` (`TurretComponentConfig`, `TurretPowerConfig`, `ShootPatternConfig`)
     - `core/src/config/block/feature/shield.rs` (`ShieldConfig`, `ShieldShapeConfig`)
     - `core/src/config/block/feature/thruster.rs` (`ThrusterConfig`)
     - `core/src/config/block/feature/power.rs` (`Generator`, `Battery`, `Consumer`, `Booster`)
     - `core/src/config/block/feature/ammo.rs` (`AmmoConfig`)
     - `core/src/config/block/feature/texture.rs` (`TextureConfig`)
     - `core/src/config/block/feature/storage.rs` (`StorageConfig`)
   - Deconstruct `core/src/config/game.rs`:
     - `core/src/config/game/game.rs` (`GameConfig`)
     - `core/src/config/game/fleet.rs` (`DefaultFleetEntry`)
   - Deconstruct `core/src/save/world/snapshot/component.rs`:
     - `core/src/save/world/snapshot/ship.rs` (`ShipSnapshot`)
     - `core/src/save/world/snapshot/block.rs` (`BlockSnapshot`, `PatternSave`)
     - `core/src/save/world/snapshot/debris.rs` (`DebrisSnapshot`)
     - `core/src/save/world/snapshot/design.rs` (`ShipDesign`)
     - `core/src/save/world/snapshot/sanitize.rs` (clamping charge, cooldown, remaining lifetime)
   - *Rationale:* Encapsulates each data kind in a self-contained unit (~50–120 lines). Changes to one block feature will not invalidate compilation caches for others.

3. **Decode-Time Validation in `RonAssetLoader`:**
   - Update `RonAssetLoader<A>` where `A: Asset + DeserializeOwned + Validate<Context = ()>` to execute `asset.validate(&())` immediately after byte deserialization.
   - Any validation error returns `io::Error(ErrorKind::InvalidData, ...)` directly into Bevy's asset pipeline.
   - *Rationale:* Eliminates two-pass desynchronization. Invalid balance files immediately put `ConfigState` into `Error` with file details; hot reload keeps the last valid configuration without polluting memory.

4. **Separation of Structural Rejection vs. Transient Sanitization:**
   - `Validate` strictly enforces structural validity: missing core blocks, duplicate IDs, invalid shape coordinates, out-of-range base values. Structural failures reject load entirely.
   - `sanitize.rs` performs lenient clamping on transient save fields (leftover battery charge, cooldown timers, remaining lifetime) at entity spawn time.
   - *Rationale:* Preserves save backward compatibility and runtime resilience while aggressively catching corrupt data structures.

## Risks / Trade-offs

- **[Risk] High file count in feature modules:** Adding 8–10 focused files could increase directory complexity.
  - *Mitigation:* Clear hierarchical folder structure (`config/block/feature/*`) where each file maps 1:1 to an enum variant or component type.
- **[Risk] Re-export breakage for existing callers:** Existing systems importing from `crate::config::block` or `crate::save::world::snapshot::component`.
  - *Mitigation:* Maintain clean public re-exports in `mod.rs` for each module so external call sites require minimal import changes.
