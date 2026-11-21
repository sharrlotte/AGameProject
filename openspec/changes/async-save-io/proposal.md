## Why

All save, load, create, continue, design-save, and WAL-checkpoint (`PRAGMA wal_checkpoint(TRUNCATE)`) disk I/O runs synchronously inside `Update` on the main thread (`core/src/save/world/system.rs:38-293`), causing visible frame hitches on every 30-second autosave and scene transition. Additionally, two lifecycle bugs exist in the current synchronous path: (1) Pause → Ship Editor (`scene/pause/mod.rs:87-104`) changes `SceneState` before `handle_save_world_event` runs, despawning all ships before capture; and (2) Play / Continue from Home (`save/world/system.rs:245`) calls `InMemoryWorldStore::list_slots()` instead of scanning `<data_dir>/saves/*.db`, always creating a new slot on fresh launch.

## What Changes

- Introduce a dedicated single background I/O worker thread (`IoWorker`) backed by a bounded `std::sync::mpsc::sync_channel` that owns the active `WorldStore` and executes all encode, `lz4_flex` compression, SQLite transactions, slot opens, design writes, and WAL flushes strictly in FIFO order off the main thread.
- Capture owned, unencoded snapshot payloads (`CapturedWorld`) on the main thread (since `&World` cannot cross threads), submit them to `IoWorker`, and track lifecycle via `SaveState::{Idle, Saving, Loading}`.
- Reject duplicate save/load requests while `SaveState` is busy, poll worker completion messages in `Update`, emit `WorldSaved` / `WorldLoaded` / `SaveFailed` messages, and ensure pending scene exits (`PendingExitAction::{None, Home, ShipEditor}`) wait for completion before transitioning scenes.
- Fix the **Pause → Ship Editor save race** by routing Pause → Ship Editor through `PendingExitAction::ShipEditor` so capture and save complete before `DespawnOnExit(SceneState::Game)` despawns live ships.
- Fix **`ContinueWorldEvent` from Home** so it scans `<data_dir>/saves/*.db` via the worker (eliminating the `InMemoryWorldStore::list_slots()` bug and the double `load_world_data` read on Continue).

## Capabilities

### New Capabilities
- `async-save-io`: Dedicated FIFO I/O worker thread, bounded command/result channels, `SaveState` machine, main-thread owned snapshot capture, and non-blocking completion polling for world saves, slot loads, creates, and flushes.

### Modified Capabilities
- `scene-lifecycle`: Defer both `Back to Home` and `Pause → Ship Editor` transitions until the async world save completes via `PendingExitAction::{None, Home, ShipEditor}`.
- `world-session-intent`: Fix `Play` / `ContinueWorldEvent` slot resolution from Home to always query persisted save slots on disk (never `InMemoryWorldStore`), and execute open/validate/read asynchronously once without double-loading.

## Non-Goals

- Changing the SQLite table schema or adding the in-memory chunk cache (handled by `chunk-storage-cache-and-layout`, which plugs into the same `IoWorker` channel).
- Changing the snapshot struct layout or co-locating component conversions (handled by `colocated-component-snapshots`).

## Impact

- `core/src/save/world/` (`mod.rs`, `component.rs`, `system.rs`, new `worker.rs` module, `store.rs`, `sqlite.rs`).
- `core/src/scene/pause/mod.rs` (`on_ship_editor` using `PendingExitAction::ShipEditor`).
- `core/src/scene/home/slots.rs` (async slot list refresh & delete).
- `docs/save.md` and `docs/scenes.md` (spec-only updates).
