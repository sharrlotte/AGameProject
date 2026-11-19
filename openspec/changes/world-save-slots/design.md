## Context

Persistence is currently split across `<data_dir>/saves/campaign.ron`, `<data_dir>/saves/battle.ron`, `<data_dir>/fleet.ron`, and `<data_dir>/designs/*.ron`. That layout was built for a single linear campaign and cannot cleanly support multiple open-world save slots, atomic commits across world state + chunks + quests, or per-slot ship designs.

## Goals / Non-Goals

**Goals:**
- Consolidate all playthrough state into **1 self-contained SQLite file per save slot** under `<data_dir>/saves/<slot_id>.db` (WAL mode, `PRAGMA user_version` schema validation).
- Provide a SOLID `WorldStore` trait with `SqliteWorldStore` (production) and `InMemoryWorldStore` (`QuickBattle` sandbox & unit tests).
- Store per-slot ship designs inside each save slot's `designs` table (seeded from built-in starter designs on New World) while keeping a separate `HomeSandbox` catalog for Home's Ship Editor and `QuickBattle`.
- Add a Save Slot Picker UI on `Home` (`Play` → list slots, Continue/Load, New World, Delete with confirmation) and keep `QuickBattle` as an unsaved sandbox.
- Remove `battle.ron`, `campaign.ron`, global `fleet.ron`, and the Main Menu Settings `Reset Mission / Campaign` section.

**Non-Goals:**
- Chunk streaming logic itself (implemented in `world-chunk-streaming`, which plugs directly into `WorldStore`'s `chunks` table).
- Removing `core/src/mission/` or implementing quests (implemented in `open-world-quests`, which plugs directly into `WorldStore`'s `quests` table).
- Moving global user preferences (`settings.ron`) into SQLite (`settings.ron` stays global RON).

## Decisions

### 1. Single-File SQLite Schema (`saves/<slot_id>.db`)
- **Choice**: Each save slot is `<data_dir>/saves/<uuid>.db` opened with `PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;` and validated against `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION` (`1`).
- **Schema**:
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
      bodies_blob   BLOB NOT NULL, -- Vec<BodySnapshot> (player fleet + nearby live entities)
      drops_blob    BLOB NOT NULL, -- Vec<DropSave>
      intel_blob    BLOB NOT NULL  -- Vec<TeamIntelSave>
  );

  CREATE TABLE designs (
      id            TEXT PRIMARY KEY, -- Uuid string
      name          TEXT NOT NULL,
      updated_at_ms INTEGER NOT NULL,
      data_blob     BLOB NOT NULL     -- postcard + lz4 ShipDesign
  );

  -- Reserved empty tables ready for open-world-quests and world-chunk-streaming:
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
- **Encoding**: Blobs use `postcard` + `lz4_flex::compress_prepend_size` for compact, fast binary serialization with zero JSON/RON parsing overhead on large worlds.
- **Atomicity**: Every save wraps `world_meta` + `active_state` (and any dirty `designs`/`quests`/`chunks`) in a single SQLite transaction (`BEGIN IMMEDIATE ... COMMIT`).
- **Slot Listing**: `list_slots(saves_dir)` scans `*.db`, opens each read-only, validates `user_version`, and queries ONLY the 1-row `world_meta` table (microseconds, never touches `active_state` or `chunks`). Incompatible/corrupted files are logged and surfaced cleanly without crashing.

### 2. `WorldStore` Trait & `WorldSession`
- **Choice**:
  ```rust
  pub enum WorldSession {
      SaveSlot { slot_id: Uuid, path: PathBuf },
      QuickBattle,
  }

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
  - `SaveSlot` uses `SqliteWorldStore`.
  - `QuickBattle` uses `InMemoryWorldStore` (zero disk I/O).
- **Rationale**: Clean dependency inversion (`AGENTS.md`), zero disk I/O in unit tests and `QuickBattle`, and seamless extension points for `quests` and `chunks`.

### 3. Per-Slot Ship Designs + Home Sandbox Isolation
- **Choice**: Introduce `DesignCatalogSource`:
  - `DesignCatalogSource::ActiveWorldDb`: When entering a Save Slot, `ShipCatalog` is hydrated from that slot's `designs` table (seeded with the 4 starter designs when a New World slot is created). Opening the Ship Editor from the Pause menu edits and persists directly to the active slot's `designs` table.
  - `DesignCatalogSource::HomeSandbox`: On `Home`, Ship Editor and `QuickBattle` use a sandbox catalog stored under `<data_dir>/sandbox_designs/` (or `.ron` sandbox) seeded from starter designs, never touching any save slot `.db`.
- **Rationale**: Each save slot is 100% self-contained in one `.db` file while Home's Ship Editor and `QuickBattle` remain immediately usable for testing.

### 4. Home Save Slot Picker & Pause Integration
- **Choice**:
  - `Home` `Play` button opens the Save Slot Picker overlay/view showing all discovered slots sorted by `updated_at_ms` descending (displaying slot name, play time, last saved time).
  - Actions: `New World` (creates a new `<uuid>.db` with random `world_seed`, seeds starter designs, enters `SceneState::Game`), `Load` (opens selected `.db` and enters `SceneState::Game`), `Delete` (2-step confirmation, deletes `.db` + `-wal`/`-shm` sidecars).
  - `Pause` menu in a `SaveSlot` session includes `Ship Editor` (edits slot designs) and auto-commits `WorldStore` on `Back to Home` (plus periodic auto-save on `auto_save_interval_secs`).
  - Remove `Reset Mission / Campaign` from Main Menu Settings (`core/src/scene/setting/`).

### 5. Module Layout (`AGENTS.md` Compliant)
```
core/src/save/world/
├── mod.rs          # WorldSavePlugin, ActiveWorldStore resource, SaveWorldEvent, LoadWorldEvent
├── component.rs    # WorldSession, SlotSummary, WorldMeta, ActiveStateSnapshot, WorldStoreError, WORLD_DB_SCHEMA_VERSION
├── store.rs        # WorldStore trait, InMemoryWorldStore, postcard + lz4 codec
├── sqlite.rs       # SqliteWorldStore, list_slots, delete_slot_files
├── snapshot.rs     # Capture/apply helpers for WorldMeta, ActiveStateSnapshot, ShipSnapshot, DebrisSnapshot
└── system.rs       # Periodic auto-save, save-on-exit, load-slot lifecycle + unit tests
```

## Risks / Trade-offs

- **Breaking Migration from `battle.ron` / `campaign.ron` / `fleet.ron`**: Old `.ron` save files are no longer read (hard cutover, left untouched on disk). → Explicitly documented in `world-save` delta spec; no silent migration hacks.
- **SQLite Sidecar Files (`-wal`, `-shm`)**: Deleting or closing a save slot must also clean up `-wal` and `-shm` files. → `SqliteWorldStore` checkpoints on `Drop`/`flush` and `delete_slot_files` removes `.db`, `.db-wal`, and `.db-shm`.
- **Ship Editor Return Target**: Entering `ShipEditor` from `Pause` inside a save slot must return to `Game` (resuming the active slot) rather than `Home`. → Track `EditorReturnTarget::{Home, Game}` resource.
