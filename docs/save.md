# Persistence Spec — SQLite Save Slots

## Module Layout

`core/src/save/` is organized by domain following the `mod.rs`, `component.rs`, `lib.rs`, `system.rs` structure:

```
core/src/save/
├── mod.rs          SavePlugin (registers DesignSavePlugin, WorldSavePlugin)
├── document.rs     RonDocument trait, DocumentError, parse/read/write_document, atomic_write, data_dir
├── design/         DesignSavePlugin, ShipCatalog, DesignCatalogSource, sandbox designs, fleet helpers
└── world/          WorldSavePlugin: single-file SQLite save slots
    ├── mod.rs      plugin wiring, re-exports
    ├── component.rs  WorldSession, SlotSummary, WorldMeta, ActiveStateSnapshot, WorldStoreError, events
    ├── store.rs    WorldStore trait, postcard + lz4 codec, InMemoryWorldStore, now_unix_ms
    ├── sqlite.rs   SqliteWorldStore, list_slots, delete_slot_files, sequential migration ladder
    ├── snapshot/   modular capture and apply subfolder (<300 lines/file)
    │   ├── mod.rs        BodySnapshot enum, module re-exports
    │   ├── component.rs  serde structs (TurretSaveState, BlockSnapshot, TransformSave, etc.)
    │   ├── capture.rs    world entity extraction (bodies, ships, debris, camera, control, drops, intel)
    │   └── apply.rs      ECS restoration and spawning (bodies, ships, debris, camera, control, drops, intel)
    └── system.rs   save/load/create lifecycle, auto-save, catalog switching, load_world_data / apply_world_data
```

## Locations

`data_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`.

```
<data_dir>/saves/<slot_id>.db          one self-contained SQLite save slot (uuid-v7 file name)
<data_dir>/saves/<slot_id>.db-wal      SQLite WAL sidecar (checkpointed on flush / drop / delete)
<data_dir>/saves/<slot_id>.db-shm      SQLite shared-memory sidecar
<data_dir>/sandbox_designs/*.ron       Home sandbox catalog (Ship Editor + QuickBattle)
<data_dir>/settings.ron / assets/config/setting.ron  global user preferences (still RON)
```

Legacy `saves/battle.ron`, `saves/campaign.ron`, `fleet.ron`, `saves/slot_*.ron`, `designs/*.ron`,
`schematics/*.json` and `world/` are never read, written, migrated, or deleted by the save system.

`WORLD_DB_SCHEMA_VERSION = 1` is stored in `PRAGMA user_version`. On open, a sequential migration
ladder runs from `found` up to `WORLD_DB_SCHEMA_VERSION`. Downgrade attempts (`found > WORLD_DB_SCHEMA_VERSION`)
fail loudly with `WorldStoreError::VersionMismatch { expected, found }` without modifying the file.
`list_slots` skips incompatible or corrupt files with a warning.

## SQLite Schema

Each slot is opened with `PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;`.

```sql
CREATE TABLE world_meta (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    name           TEXT NOT NULL,
    seed           INTEGER NOT NULL,
    play_time_secs REAL NOT NULL,
    created_at_ms  INTEGER NOT NULL,
    updated_at_ms  INTEGER NOT NULL,
    camera_blob    BLOB NOT NULL,
    control_blob   BLOB NOT NULL,
    rng_blob       BLOB NOT NULL
);
CREATE TABLE active_state (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    bodies_blob   BLOB NOT NULL,
    drops_blob    BLOB NOT NULL,
    intel_blob    BLOB NOT NULL
);
CREATE TABLE designs (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    data_blob     BLOB NOT NULL
);
CREATE TABLE quests (
    quest_id      TEXT PRIMARY KEY,
    status        TEXT NOT NULL,
    progress_blob BLOB NOT NULL
);
CREATE TABLE chunks (
    x             INTEGER NOT NULL,
    y             INTEGER NOT NULL,
    visited       INTEGER NOT NULL,
    enemy_count   INTEGER NOT NULL,
    data          BLOB NOT NULL,
    PRIMARY KEY (x, y)
);
```

`quests` and `chunks` are reserved empty tables for `open-world-quests` and `world-chunk-streaming`.

## Encoding and Atomicity

Binary payloads use `postcard` followed by `lz4_flex::block::compress_prepend_size`; decoding uses
`decompress_size_prepended` then `postcard::from_bytes`. Corrupt blobs return
`WorldStoreError::Decode` (fail loud, never fabricate state).

`save_world` writes `world_meta`, `active_state`, and any dirty designs inside a single
`BEGIN IMMEDIATE … COMMIT` transaction. A rollback leaves the previously committed state intact.
`flush()` runs `PRAGMA wal_checkpoint(TRUNCATE)`; `Drop` also checkpoints with a warning on failure.
`delete_slot_files` removes `.db`, `.db-wal`, and `.db-shm`.

`list_slots` opens each `*.db` read-only, validates `user_version`, and selects only the one
`world_meta` row — never `active_state`, `designs`, `quests`, or `chunks`. Results are sorted by
`updated_at_ms` descending and feed the Home Save Slot Picker (name, play time, last-saved time).

## WorldStore Abstraction

```rust
pub trait WorldStore: Send + Sync {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError>;
    fn load_active_state(&self) -> Result<ActiveStateSnapshot, WorldStoreError>;
    fn save_world(&mut self, meta: &WorldMeta, active: &ActiveStateSnapshot) -> Result<(), WorldStoreError>;
    fn list_designs(&self) -> Result<Vec<ShipDesign>, WorldStoreError>;
    fn save_design(&mut self, design: &ShipDesign) -> Result<(), WorldStoreError>;
    fn delete_design(&mut self, id: Uuid) -> Result<(), WorldStoreError>;
    fn flush(&mut self) -> Result<(), WorldStoreError>;
}
```

- `SqliteWorldStore` — production single-file save slot. `Connection` is wrapped in `Mutex` so the
  store is `Send + Sync` and can live in the `ActiveWorldStore` ECS resource.
- `InMemoryWorldStore` — `QuickBattle` and unit tests. Zero disk I/O; `WorldSession::QuickBattle`
  never creates, reads, overwrites, or deletes any `<data_dir>/saves/*.db` file.

## Session Lifecycle

`WorldSession` is `SaveSlot { slot_id, path }` or `QuickBattle`. `ActiveWorldStore` holds the live
store as `Option<Box<dyn WorldStore>>`.

- **New World**: creates `<slot_id>.db`, seeds starter designs into its `designs` table, sets
  `DesignCatalogSource::ActiveWorldDb`, hydrates `ShipCatalog` and `DeployPayload` from the slot,
  enters `SceneState::Game` with a fresh mission.
- **Load**: opens the selected `.db`, hydrates catalog/fleet from the slot, sets `PendingLoad`,
  transitions `Loading → Game`, and `spawn_loaded_world` applies the snapshot.
- **Auto-save**: `SaveWorldEvent` on `auto_save_interval_secs` while `BattlePhase::Active` and
  `PauseState::Playing`; also on Pause `Back to Home` before the scene transition. Save is followed
  by `flush()` so WAL sidecars are checkpointed.
- **QuickBattle**: `StartQuickBattleEvent` installs `InMemoryWorldStore` and `HomeSandbox` catalog;
  no save-slot disk access.
- **Ship Editor return**: `EditorReturnTarget::{Home, Game}`; Pause → Ship Editor sets `Game` so the
  active slot resumes instead of returning Home.

## Design Catalog Isolation

`DesignCatalogSource::{ActiveWorldDb, HomeSandbox}` selects where Ship Editor writes go:

- Inside a save slot, `DesignSaveRequested` is persisted into that slot's `designs` table.
- From Home / QuickBattle, it is persisted to `<data_dir>/sandbox_designs/*.ron`.

`ShipCatalog` is the in-memory cache; `persist_design_request` owns the write and upserts the cache.
Saving a design with an existing name replaces the stale row instead of duplicating it.

## Snapshot Capture / Apply

`save/world/snapshot/` (`capture.rs` and `apply.rs`) ports the single-ship, single-debris, drop, camera, control, RNG, and
`TeamIntel` helpers from the former battle save:

- **Capture** reads directly from `&World` (`BodySnapshot::capture`, `capture_drops`, `capture_camera`,
  `capture_control`, `capture_intel`).
- **Apply** is two-pass: spawn ships/debris and restore block state, then resolve `Uuid` references
  through `ship_map` for `Brain`, `TeamIntel`, camera follow and controlled ship; drops are spawned
  with visible sprites and `DespawnOnExit(SceneState::Game)`.
- Health, battery `Charge`, `TurretState`/`TurretCharge`/barrel rotation, `AmmoState`, weapon pattern
  timers, cargo, skill, doctrine, and `shed_boosters` are preserved.
