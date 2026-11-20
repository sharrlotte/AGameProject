## Why

A whole-codebase audit found invalid states reachable in normal play: the editor "Play" path can silently overwrite a save slot, an out-of-range `Team` panics on a bit shift, `ShipStats.health` goes stale so AI never retreats, and independent Pause/Settings/SlotPicker states strand the game behind modals or with frozen time. Mission-layer findings are excluded because `core/src/mission/` is being removed by `open-world-quests`. This change must land **before** `world-chunk-streaming` and `open-world-quests` because it changes the world-save schema and the `WorldStore` trait that both build on.

## What Changes

- Replace `PendingLoad(bool)` with an explicit `PendingWorld` resource (`None`, `Restore { slot_id }`) so entering `SceneState::Game` always states whether the saved world is being restored.
- Redefine "Play" on Home **and** in the editor: continue the last save (active slot first, else most recently updated slot); if none exists, create a new slot automatically. Sandbox/QuickBattle stays unsaved. A failed continue shows a notice and stays in the current scene — never silently replaces the save.
- Add a "Worlds" button on Home that opens the slot picker, so loading a different or named slot remains reachable.
- **BREAKING**: bump `WORLD_DB_SCHEMA_VERSION` 1 → 2. Drop `WorldMeta.rng_blob`; `meta.seed` becomes the single source of truth. No migration is written (project is still in dev, no save data to preserve); v1 slots are not loadable.
- Add `list_slots()` to the `WorldStore` trait (in-memory returns its active slot).
- Introduce a fail-loud validation gate at every load boundary (`read_designs_dir`, SQLite design decode, `load_world_data`) returning typed errors; reject the whole load on structural corruption and show a notice.
- Make `Team` constructible only through validated paths: private bounded id (`< 16`), deserialization via `TryFrom<u8>`, computed color accessor, no `Reflect`.
- Recompute `ShipStats.health` from per-block `Health` in `SimulationSet::Stats` before AI systems.
- Replace `PauseState`, `SettingState`, and `SlotPickerState` with a single `ModalStack` resource; Escape pops the top modal, `OnExit(SceneState::Game)` clears the stack and unpauses virtual time. Rewrite all `run_if(in_state(...))` gates.
- Enforce `BlockRegistry::validate_ship` on all design load/spawn paths; make `BlockGrid.blocks`/`cells` private and rebuild `cells` from `blocks` in one place.
- Change `BlockSnapshot`'s four optional pattern fields to a single discriminated `PatternSave` enum.
- Extend config validation (`AiConfig::validate` / `.validated()` pattern) to `ShootPattern` and `Thruster` numeric fields; reject negative damage/width/lifetime.
- Collapse `TurretState`'s five parallel fields into one enum and remove dead `shoot_at`; derive cargo `used` from items; clean up stale beam/entity handles; make `Skill`/`Throttle`/`BoundingBox` fields private with validated setters; reset all `EditorState` fields on `OnEnter(SceneState::ShipEditor)`.

## Capabilities

### New Capabilities
- `ui-modal-stack`: one owner for Pause / Settings / SlotPicker modals, Escape-to-pop, scene-exit clearing, and virtual-time pausing.
- `save-integrity`: typed fail-loud validation at all load boundaries, schema v2, single-source seed, rejection UX.
- `world-session-intent`: explicit world transition intent, "continue last save, else create" Play semantics, slot resolution, Worlds-button access, failed-continue behavior.

### Modified Capabilities
- `config-validation`: numeric validation for `ShootPattern` and `Thruster` definitions.
- `editor-ui`: Play no longer deploys the current grid (Deploy retains that role), Back label, complete editor state reset on entry.
- `world-save-slots`: Worlds button opens the picker (Play continues instead), any schema-version mismatch fails loudly, auto-created slot naming.
- `scene-lifecycle`: `PendingWorld` replaces `PendingLoad`, Home may host the Worlds button while Pause stays free of save/load UI, modal stack cleared on leaving Game.

> Compliance-only fixes with no spec delta: `ShipStats.health` tracking block health and spawn-time layout validation are already required by `block-grid`; bounded `Team` ids are already required by `config-validation`. They are implementation work in `tasks.md`, not new requirements.

## Impact

- **Code**: `core/src/main.rs`, `core/src/ui/**`, `core/src/resource/**`, `core/src/scene/**`, `core/src/save/**`, `core/src/content/**`, `core/src/config/**`, `core/src/system/**`.
- **APIs**: `WorldStore` gains `list_slots`; `WorldMeta` loses `rng_blob`; `PendingLoad` is replaced by `PendingWorld`; three Bevy `States` are removed; `ShipDesign::spawn_with_id` becomes fallible with a typed error.
- **Dependencies**: none new.
- **Sequencing**: must be applied before `world-chunk-streaming` and `open-world-quests` to avoid conflicting edits to `WorldStore`, `sqlite.rs`, and `main.rs`.
