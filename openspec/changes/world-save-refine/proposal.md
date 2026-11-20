## Why

The `world-save-slots` implementation introduced a solid SQLite-backed persistence layer, but a code review identified correctness bugs (non-atomic design saves, premature success notifications), code quality violations (duplicated helpers, files 2–3× over the 300-line limit), missing forward compatibility (no migration ladder for schema upgrades), and an overly large system function that mixes I/O, decode, and ECS mutation in one monolith.

## What Changes

- **FIX**: `SqliteWorldStore::save_design` made fully atomic — stale deletion and upsert run inside one `IMMEDIATE` transaction with a single mutex lock
- **FIX**: `ShipSavesChanged` message is now emitted by `persist_design_request` only after successful persistence, never optimistically from the editor
- **FIX**: Duplicate `now_unix_ms()` helper consolidated into `store.rs` as a single shared `pub fn`
- **ADD**: Sequential schema migration ladder in `sqlite.rs` — `open()` calls `migrate()` instead of `check_version()`; v0→v1 step defined; downgrades rejected; pattern is extensible for v2+
- **REFACTOR**: `spawn_loaded_world` decomposed into `load_world_data()` (I/O + decode, returns `WorldLoadData`) and `apply_world_data()` (ECS mutation) helper functions within `system.rs`
- **REFACTOR**: `snapshot.rs` (860 lines) split into `snapshot/` subfolder: `component.rs` (serde structs), `capture.rs` (capture functions), `apply.rs` (apply functions), `ship.rs` (ShipSnapshot + BrainSave + TacticSave), `debris.rs` (DebrisSnapshot), `mod.rs` (re-exports + BodySnapshot enum)
- No behavior changes visible to the player; no breaking changes to saved data format

## Capabilities

### New Capabilities

- `world-save-migration`: Sequential schema migration ladder for SQLite save slots — defines the pattern for future schema changes

### Modified Capabilities

- `world-save-slots`: Requirements change — design saves MUST be atomic (single transaction), `ShipSavesChanged` MUST only emit on confirmed persistence, schema opening MUST attempt migration before rejecting; file structure MUST comply with 300-line rule

## Impact

- **Files removed**: `core/src/save/world/snapshot.rs`
- **Files added**: `core/src/save/world/snapshot/` (6 files), no other new public APIs
- **Files modified**: `sqlite.rs`, `store.rs`, `system.rs`, `save/design/system.rs`, `save/design/component.rs`, `scene/editor/save.rs`, `scene/editor/mod.rs`
- **Dependencies**: No new crate dependencies
- **Tests**: Existing unit tests remain valid; migration ladder tests added to `sqlite.rs`
