## Context

Today, `core/src/save/world/snapshot/` manually mirrors and maps gameplay components from outside their owning modules:
- `Tactic` (7 variants in `content/ship/ai/component.rs:207-233`) is manually duplicated as `TacticSave` (`snapshot/ship.rs:33-59`), mapped in `capture.rs:20-52`, and mapped back in `apply.rs:31-56`.
- `BlockSnapshot` (`snapshot/block.rs:131-236`) reaches into `Health`, `Charge`, `TurretState`, `TurretBarrel`, `TurretCharge`, `AmmoState`, and 4 weapon pattern states — yet silently omitted `Shield` (barrier HP, `ShieldState`) and `Disabled` (booster shedding), even though `docs/save.md:188-189` claimed `Disabled` was saved.
- `BlockSnapshot::validate` (`block.rs:68-84`) and `DebrisSnapshot::validate` (`debris.rs:26-28`) reject negative/non-finite `battery_charge`, `turret.charge`, and `remaining_secs`, contradicting `openspec/specs/save-integrity/spec.md:28-33` (which mandates clamping non-structural transient values with a warning) and leaving `sanitize.rs` partially unreachable or completely dead (`sanitize_remaining_secs`).
- `ShipSnapshot::restore` constructs a temporary `ShipDesign` and calls `spawn_with_id`, re-running `registry.validate_ship` a second time after `load_world_data` already validated it. Meanwhile `world/chunk/system.rs:287` calls `apply_bodies` without validating chunk snapshots at all.

## Goals / Non-Goals

**Goals:**
- Co-locate each persistable component's `Save` DTO, `to_save`, and `from_save` (with field-level clamping/sanitization) in the owning module via a lean, typed `Snapshot` trait.
- Keep aggregate snapshots (`BlockSnapshot`, `ShipSnapshot`, `DebrisSnapshot`) as thin, readable assemblers with zero duplicated component-mapping logic.
- Persist full runtime fidelity including `Shield` (`ShieldState` + barrier HP), booster `Disabled` shedding, `TurretState::{Idle, Tracking, Ready}`, and `Brain.think_timer` remaining seconds.
- Unify structural validation vs. non-structural sanitization so both slot load and chunk reload follow `save-integrity` identically, without double `validate_ship`.

**Non-Goals:**
- Introducing a one-size-fits-all ECS entity trait with dummy `Entity::PLACEHOLDER` arguments (the flaw of the archived `CaptureSnapshot`/`RestoreSnapshot`).
- Using `DynamicScene` / Bevy reflection for persistence.

## Decisions

### 1. Lean Typed `Snapshot` Trait in a Leaf Module
- **Choice**: Define a small trait (e.g., in `core/src/save/snapshot.rs` or `core/src/validate/snapshot.rs` to keep module dependencies strictly one-way, `content/* → validate` and `save → content/*`):
  ```rust
  pub trait Snapshot: Sized {
      type Save: Serialize + DeserializeOwned;
      type CaptureCtx<'a>;
      type RestoreCtx<'a>;

      fn to_save(&self, ctx: &Self::CaptureCtx<'_>) -> Self::Save;
      fn from_save(save: Self::Save, ctx: &Self::RestoreCtx<'_>) -> Self;
  }
  ```
  - Simple value components (`Health`, `Charge`, `AmmoState`, `Shield`, `Transform`, `Velocity`, `ShipStats` cargo) use `CaptureCtx<'a> = ()` and `RestoreCtx<'a> = Uuid` (block/ship ID for warning logs during sanitization).
  - Reference-carrying components (`Tactic`, `Brain`, `TeamIntel`) use `CaptureCtx<'a> = &'a World` (or `&'a dyn Fn(Entity) -> Option<Uuid>`) and `RestoreCtx<'a> = &'a HashMap<Uuid, Entity>`.
  - Each owning module defines its `Save` struct and `impl Snapshot for MyComponent` right beside the runtime type — adding a field to `Brain` or `Shield` is a single-module edit, and the compiler immediately flags `to_save`/`from_save` if a field is missed!
- **Rationale**: Solves the 4-file duplication bug without the `Entity::PLACEHOLDER` anti-pattern of the old `CaptureSnapshot`/`RestoreSnapshot` traits and without coupling saves to Bevy ECS reflection layout.
- **Alternative Considered**: *Keep all DTOs in `save/world/snapshot/` and rely on manual discipline* — already resulted in silently missing `Shield`, `Disabled`, `TurretState::Tracking`, and `Brain.think_timer` remaining time.

### 2. Full Fidelity Additions (`ShieldSave`, `Disabled`, `TurretSaveState`, `Brain`)
- **Choice**:
  - `ShieldSave { state: ShieldState, barrier_hp: f32 }` captured via `Shield::to_save` (reading barrier `Health` from `&World`) and restored on `Shield` + initial barrier HP on observer spawn (`unified-child-entity-observers`).
  - `BlockSnapshot.disabled: bool` captures explicit `Disabled` on live ship blocks (booster shedding) and inserts `Disabled` on restore.
  - `TurretSaveState` replaces `active: bool` with an explicit `TurretModeSave::{Idle, Tracking { aim_point }, Ready { aim_point }}` so `Tracking` round-trips faithfully.
  - `BrainSave` stores both `timer_duration_secs` and `timer_remaining_secs` (clamped cleanly in `Brain::from_save`).

### 3. Clean Validation vs. Sanitization Contract
- **Choice**:
  - **Structural `Validate`** (rejects entire load atomically): unknown block types, nil/duplicate IDs, overlapping cells, zero/multiple cores, disconnected layout, invalid `team >= 16`, invalid `skill` outside `1..=10`, non-finite `Transform`/`Velocity` coordinates, dangling ship `Uuid` references.
  - **Non-structural sanitization** (clamps + emits `warn!` inside `from_save`, never rejects load): `health` / `max_health`, `battery_charge`, `turret.charge`, `barrier_hp`, `cooldown_remaining`, `remaining_secs`, `cargo.used` vs manifest sum, `Brain` timers and `strafe_sign`.
  - Remove the contradictory hard-reject checks for `battery_charge`, `turret.charge`, and `remaining_secs` from `BlockSnapshot::validate` / `DebrisSnapshot::validate` so `sanitize` actually enforces `save-integrity`.
  - Add an internal `spawn_validated_with_id` helper on `ShipDesign` (or pass pre-validated `BlockGrid`) so `ShipSnapshot::restore` does not run `registry.validate_ship` twice.
  - Call `ShipSnapshot::validate` / `DebrisSnapshot::validate` in `stream_chunks::load_in_range` before spawning restored chunk bodies.

## Risks / Trade-offs

- **One-way module dependency direction** → Placing the tiny `Snapshot` trait in `core/src/validate/` (zero dependency on `save/` or `content/`) ensures `content/*` depends only on `validate`, and `save/` depends on `content/*`, preserving `module-boundaries/spec.md` with zero cycles.
- **Schema compatibility** → Adding `shield`, `disabled`, and `TurretModeSave` changes `BlockSnapshot`'s binary `postcard` layout, requiring a `WORLD_DB_SCHEMA_VERSION` bump (hard cutover, no legacy migration per project spec).
