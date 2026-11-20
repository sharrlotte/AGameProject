## Context

An audit across `core/src` found invalid states reachable through ordinary UI flows and through load boundaries. The codebase already uses enums well (`SceneState`, `Tactic`, `ShootPattern`, `Laser` phase states), but several areas encode state as parallel fields or independent resources:

- `PendingLoad(bool)` cannot distinguish "restore a save" from "spawn a fresh session", so the editor's Play button re-enters `Game` with a stale `SaveSlot` session and autosave overwrites the world.
- `PauseState`, `SettingState`, and `SlotPickerState` change independently of `SceneState`, producing frozen time over Home and live simulation behind a settings modal.
- Load boundaries trust deserialized data: `Team` can be built with `id >= 16` via serde/Reflect (`1 << (16 + id)` then panics), `ShipDesign` layouts are never validated outside the editor, and `BlockSnapshot` can hold four mutually exclusive shoot patterns at once.
- `ShipStats.health` and `CargoSave.used` are caches that drift from their sources; `block-grid` and `config-validation` already require the correct behavior, so these are compliance gaps rather than new requirements.

`core/src/mission/` is excluded: `open-world-quests` deletes it. Two active changes (`world-chunk-streaming`, `open-world-quests`) also modify `WorldStore` and `main.rs`, so this change lands first.

## Goals / Non-Goals

**Goals:**
- Make the save/continue flow deterministic and incapable of overwriting a save with a fresh session.
- Give modal UI one owner, so no combination of scene + modal + virtual time is contradictory.
- Make invalid persisted data fail loudly at the boundary with a typed error and a player-facing message.
- Make `Team` impossible to construct out of range; make ship health and cargo derive from their sources.
- Bring code into compliance with already-archived `block-grid` and `config-validation` requirements.

**Non-Goals:**
- Any change to `core/src/mission/`, quest design, or chunk streaming.
- Writing a v1 → v2 save migration (no save data to preserve; project still in dev).
- Adding a persistence prefs file for "last played" — last save is resolved from the active session or slot metadata.
- Making the sandbox/QuickBattle persistent. It remains in-memory only.

## Decisions

### 1. Explicit `PendingWorld` instead of `PendingLoad(bool)`
`PendingWorld::{None, Restore { slot_id }}` is set only after slot resolution and validation succeed. `None` means "fresh skirmish" (QuickBattle). **Alternative:** keep a bool plus parallel slot-id/error fields — rejected because it recreates exactly the parallel-field problem. The quest-era game-spawn path will branch on this single value.

### 2. Play semantics: continue last save, else create
Resolution order: active `WorldSession::SaveSlot` → most recently updated slot from `WorldStore::list_slots()` → auto-created slot with a generated name and starter fleet. If validation of the resolved slot fails, show a notice and stay in the current scene; never auto-create a replacement. **Alternative:** Play as sandbox (rejected — user wants campaign continuity). **Alternative:** fall back to creating a new save on failure (rejected — silently abandons corrupt data).

### 3. "Worlds" button owns the slot picker
Home's Play no longer opens the picker; a dedicated Worlds button does. **Alternative:** reach the picker from Pause only — rejected because it removes cold-start access. This keeps all existing picker code intact and only changes its entry point.

### 4. Fail-loud validation with a single gate
`ShipSnapshot`, `BlockSnapshot`, and `ShipDesign` gain `validate() -> Result<(), InvalidData>`; `read_designs_dir`, SQLite design decode, and `load_world_data` call it immediately after decode. Structural faults (overlaps, duplicate ids, invalid team, multiple patterns) reject the whole load and show a notice. Sliders/clampables (charge, cooldown) are normalized with a warning. **Alternative:** normalize everything — rejected because it hides corruption. **Alternative:** validate inside `Deserialize` — rejected because serde has no path to the `BlockRegistry` needed for geometry checks.

### 5. `meta.seed` is the single source of truth; schema v2
Drop `WorldMeta.rng_blob`; `slot list` reads `meta.seed` directly; `GameRng` is set on every create/load path. Bump `WORLD_DB_SCHEMA_VERSION` to 2; any mismatch (lower or higher) returns `VersionMismatch`. **Alternative:** keep the blob as authoritative — rejected because slot listing would decode a blob per slot. No migration is written; v1 files are rejected (dev-only data).

### 6. `ModalStack` replaces three Bevy `States`
`Modal::{Pause, Settings, SlotPicker}` in a `Resource` stack; Escape pops the top; `OnExit(SceneState::Game)` clears the stack and unpauses `Time<Virtual>`. All `run_if(in_state(...))` gates become a `modal_is(Modal)` predicate. **Alternative:** keep the States as mirrors — rejected because duplicated state reintroduces the bug class. Consequence: `DespawnOnExit(PauseState)` / `DespawnOnExit(SettingState)` / `DespawnOnExit(SlotPickerState)` become explicit despawn-on-change, and `DebugAuditPlugin` must audit the new ownership.

### 7. `Team` is a validated newtype
Private `u8` id, `TryFrom<u8>` serde bridge, computed `color()` accessor, no `Reflect`. `assert!` is removed from the construction path. **Alternative:** keep Reflect and reconcile invalid teams each tick — rejected because it corrects rather than prevents invalid state; team ids are authored in config, not live-edited.

### 8. Compliance fixes for existing requirements
`ShipStats.health` is recomputed from block `Health` in `SimulationSet::Stats` before AI (satisfies `block-grid`'s "Health cache follows block health"). `validate_ship` is enforced inside `ShipDesign::spawn_with_id`, which becomes fallible; `BlockGrid.blocks`/`cells` become private and `cells` is rebuilt only from `blocks` (satisfies `block-grid`'s "Spawn and editor validate ship layouts"). Cargo `used` is derived from items. These need no spec delta.

### 9. Snapshot pattern becomes a discriminated union
`BlockSnapshot`'s four pattern options become `pattern: Option<PatternSave>` with `Single`/`Burst`/`Laser`/`Pulling` variants. Schema v2 carries this; no v1 reader. Turret runtime state is collapsed from five fields into one enum; dead `shoot_at` and `LaserBeamPatternState.targets` are removed.

### 10. Config numeric validation
The `AiConfig::validate()` / `.validated()` pattern is extended to `ShootPattern` and `Thruster` definitions: `fire_rate > 0`, `speed >= 0`, `damage >= 0`, `lifetime >= 0`, `width > 0`, `length > 0`, `amount > 0`, `piercing >= 0`. Violations produce the config error screen naming the block.

## Risks / Trade-offs

- **v1 save slots become unloadable** → Intentional; no migration. Notice must name the version mismatch rather than silently hiding the slot.
- **Removing Reflect from `Team`** → WorldInspector can no longer edit teams; mitigation: teams remain editable in RON config, which is validated on load.
- **`ModalStack` replaces Bevy States** → All `DespawnOnExit` usages tied to those States and the debug audit must be converted in the same change, or UI entities leak.
- **Three active changes edit the same files** → Land this change first; `world-chunk-streaming` and `open-world-quests` then build on schema v2 and `PendingWorld`.
- **Fail-loud rejection of partially corrupt saves** → Player cannot load a damaged world; mitigation: the notice names the exact reason and Worlds remains available to pick another slot.
- **Play auto-creating a slot** → Convenience vs. surprise; mitigation: generated name is explicit in the slot list and no sandbox data is ever copied in.

## Migration Plan

1. Apply this change on a clean tree before the other two active changes.
2. Bump `WORLD_DB_SCHEMA_VERSION` to 2; remove `rng_blob` from `WorldMeta` and its encode/decode sites.
3. Any existing v1 `.db` files are reported as version-mismatched and left untouched. Delete them manually if desired.
4. Rollback: revert the change; v2 files then report a version-mismatch error (the binary expects v1). No destructive conversion runs in either direction.

## Open Questions

None. All design decisions were resolved with the user before this proposal was created.
