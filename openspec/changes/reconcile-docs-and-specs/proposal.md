## Why

Multiple `/docs` specifications and archived `openspec/specs/` baselines have drifted far behind the actual codebase:
- `docs/architecture.md` still documents deleted `PauseState`, `SettingState`, `BattlePhase`, `SlotPickerState`, `PendingLoad`, `WorldSession`, `ActiveWorldStore`, `DeployPayload.current_ship`, `mission/`, `AI_THINK_RATE_HZ`, `load_current_ship`, `death::despawn_dead`, `update_ship_behavior`, and `"deny"` workspace lints that are actually `"warn"`.
- `docs/save.md` contradicts itself on schema version (v3 vs v2), `WorldSession`/`ActiveWorldStore`/`PendingLoad`/`PendingWorld` vs `WorldContext`, `&mut self` vs `&self` `WorldStore`, and migration ladders that were deleted.
- `docs/scenes.md` still describes `Play` opening `SlotPickerState::Open`, `DeployPayload.current_ship`, `PauseState`, and `SettingState`.
- `openspec/specs/world-save/spec.md` still requires "RON is the only serialization format" alongside `battle.ron`, `campaign.ron`, `WorldSave`, and `BattlePhase::Ended`; `world-save-migration/spec.md` still mandates schema migration forking; `world-save-slots/spec.md` still says schema version 2; `code-foundation/spec.md` still mandates deleted `CaptureSnapshot`/`RestoreSnapshot` traits; and `open-world-quests/spec.md` still references deleted `WorldSession`.

Stale specs mislead both human contributors and AI agents into building against dead APIs.

## What Changes

- Reconcile `docs/architecture.md`, `docs/save.md`, `docs/scenes.md`, and `docs/quests.md` to strictly describe the current implemented architecture (spec only, zero implementation notes or historical diffs).
- Reconcile drifted delta specs across `world-save`, `world-save-slots`, `world-save-migration`, `code-foundation`, `collapsed-features`, and `open-world-quests` so `openspec/specs/` matches live reality before new architectural proposals land.
- Remove dead leftover migration notice (`save/world/system.rs:140-142`) that can never trigger after migration-ladder removal.

## Capabilities

### New Capabilities
*(none)*

### Modified Capabilities
- `world-save`: Remove contradictory RON-only world save / `battle.ron` / `campaign.ron` / `BattlePhase` / `WorldSave` requirements and align with SQLite v3 + `ActiveStateSnapshot` + `WorldMeta` + sandbox RON designs.
- `world-save-slots`: Update schema version from 2 to 3 and include `quests` in the atomic save transaction.
- `world-save-migration`: Replace removed sequential fork migration requirement with strict hard-cutover `VersionMismatch` rejection matching `SqliteWorldStore::open`.
- `code-foundation`: Remove obsolete `CaptureSnapshot`/`RestoreSnapshot` mandate and stale banned-type entries contradicted by live modules (`TurretSaveState`, `Ammo`).
- `collapsed-features`: Reconcile current child-entity layout (`Visual`, extra textures, turret barrel, shield `Barrier`, overdrive `Field`) with reality.
- `open-world-quests`: Replace deleted `WorldSession` reference with `WorldContext` and align discovery acceptance scenarios (`Auto` vs `Manual`).

## Non-Goals

- Implementing any new runtime feature or refactor (handled by the other 12 dedicated proposals).
- Adding implementation notes, changelogs, or commentary into `/docs` (strictly spec-only per `AGENTS.md`).

## Impact

- `docs/architecture.md`, `docs/save.md`, `docs/scenes.md`, `docs/quests.md`.
- `openspec/specs/{world-save,world-save-slots,world-save-migration,code-foundation,collapsed-features,open-world-quests}/spec.md` (via archive).
- `core/src/save/world/system.rs` (remove dead 3-line migration notice check).
