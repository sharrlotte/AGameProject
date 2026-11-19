## Context

Currently, `spawn_mission` spawns every enemy ship in the mission simultaneously into Bevy ECS and Rapier, and all ships + debris remain permanently simulated regardless of distance from the player. `BattleSave` (`SAVE_SCHEMA_VERSION = 7`) serializes the entire live world into a single in-memory `battle.ron` file.

As the game scales toward very large procedurally generated maps, keeping distant enemies and debris live wastes physics/AI CPU cycles, and parsing an entire monolithic save file into memory will not scale. At the same time, creating thousands of loose per-chunk `.ron` files per save complicates save management and cross-chunk queries.

## Goals / Non-Goals

**Goals:**
- Partition world space into `50,000 × 50,000` storage chunks (`ChunkCoord(IVec2)`), cleanly separating *spatial storage partitioning* from *active simulation distance*.
- Stream **enemy ships** (`ShipSnapshot`) and **detached debris** (`DebrisSnapshot`) in/out of live ECS using distance hysteresis (`load_radius < despawn_radius`, loaded from `.ron`) across the player's 3×3 chunk neighborhood, eliminating chunk-border pop-in and thrashing.
- Keep player/allied ships always live in ECS.
- Abstract persistence behind a SOLID `ChunkStore` trait with:
  - `SqliteChunkStore` backed by a single bundled SQLite file (`rusqlite` with `bundled`, WAL mode, `PRAGMA user_version` schema versioning) storing compressed chunk payloads (`postcard` + `lz4_flex`) alongside indexed `enemy_count` metadata.
  - `InMemoryChunkStore` for deterministic unit testing with zero disk I/O.
- Ensure `check_battle_outcome` counts `live_enemy_ships + chunk_store.total_enemy_count()` so despawning out-of-range enemies never triggers a false Victory.
- Integrate `ChunkStore` cleanly with `save/battle` (`SaveWorldEvent`, `LoadWorldEvent`, `clear_battle_save`, fresh mission start).

**Non-Goals:**
- Procedural world/encounter generation itself (this change establishes the `ChunkCoord` + `ChunkStore` foundation so future procedural generation can populate unvisited chunks on `load() == Ok(None)`).
- Off-screen background simulation of unloaded enemies (unloaded enemies and debris freeze in place at their saved snapshot state until reloaded).
- Streaming player-team/allied ships or transient projectiles/drops.

## Decisions

### 1. Spatial Storage (50k × 50k Chunks) + Distance Hysteresis (`load_radius` / `despawn_radius`)
- **Choice**:
  - `ChunkCoord::from_world(pos, chunk_size)` maps world coordinates via `(pos / chunk_size).floor().as_ivec2()` with default `chunk_size = 50_000.0`.
  - A periodic streaming system (`SimulationSet::Cleanup` in `FixedUpdate` or throttled timer from `chunk.ron`, e.g. `0.25s`) resolves the player anchor position (same player-ship centroid / controlled ship / camera fallback):
    1. **Offload (Despawn & Save)**: Any live non-player `Ship` or `Debris` farther than `despawn_radius` (e.g. `10_000.0`) from the nearest live player-team ship is captured as `ShipSnapshot` / `DebrisSnapshot`, appended to its `ChunkCoord` bucket in `ChunkStore`, and despawned (along with its child blocks).
    2. **Load (Spawn)**: For each `ChunkCoord` in the 3×3 neighborhood of each player-team ship's chunk, inspect stored chunk entries; any `ShipSnapshot` or `DebrisSnapshot` within `load_radius` (e.g. `8_000.0`) is removed from the chunk record, saved back (or deleted if empty), and spawned into live ECS using the shared snapshot spawn helpers from `save/battle`.
  - **Multi-ally awareness**: Measuring distance to the *nearest* live player-team ship ensures enemies near an allied scout stay live and don't vanish in front of an ally.
- **Rationale**: Prevents chunk-border pop-in even when the player sits right on a `50,000` boundary, keeps live Rapier/AI entity count bounded, and prevents spawn/despawn thrashing via `despawn_radius - load_radius` hysteresis.
- **Alternatives considered**: Strict 3×3 chunk loading without distance check (keeps 150k × 150k live — far too wide); 10k × 10k chunks (25× more chunk keys for open-world scale).

### 2. Single-File SQLite Backend (`rusqlite` bundled) Behind `ChunkStore` Trait
- **Choice**:
  ```rust
  pub trait ChunkStore: Send + Sync {
      fn load(&self, coord: ChunkCoord) -> Result<Option<ChunkData>, ChunkStoreError>;
      fn save(&mut self, coord: ChunkCoord, data: &ChunkData) -> Result<(), ChunkStoreError>;
      fn remove(&mut self, coord: ChunkCoord) -> Result<(), ChunkStoreError>;
      fn total_enemy_count(&self) -> Result<usize, ChunkStoreError>;
      fn clear(&mut self) -> Result<(), ChunkStoreError>;
      fn flush(&mut self) -> Result<(), ChunkStoreError>;
  }
  ```
  - `SqliteChunkStore` stores all chunks in `<data_dir>/saves/battle_chunks.sqlite` using WAL mode (`PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;`) and `PRAGMA user_version = CHUNK_SCHEMA_VERSION`.
  - Schema:
    ```sql
    CREATE TABLE IF NOT EXISTS chunks (
        x           INTEGER NOT NULL,
        y           INTEGER NOT NULL,
        enemy_count INTEGER NOT NULL,
        data        BLOB NOT NULL,
        PRIMARY KEY (x, y)
    );
    ```
  - `ChunkData { enemies: Vec<ShipSnapshot>, debris: Vec<DebrisSnapshot> }` is serialized with `postcard` and compressed with `lz4_flex::compress_prepend_size`.
  - `total_enemy_count()` executes `SELECT COALESCE(SUM(enemy_count), 0) FROM chunks` (or maintains a cached count synchronized on `save`/`remove`/`clear` and verified on open), giving instant Victory checks without deserializing any blobs.
- **Rationale**: Single file per save, O(1) random-access chunk load/save, crash-safe ACID transactions, tiny disk footprint, and future open-world features (factions, stations, sectors, fog-of-war) can add SQL tables to the same file without breaking chunk storage.
- **Alternatives considered**: Embedded in `battle.ron` (parses whole world into memory — won't scale); per-chunk `.ron` files (thousands of files per save); `redb` (pure Rust KV, but less extensible for cross-chunk queries and secondary tables); custom region files (high maintenance/corruption risk).

### 3. Reusing `ShipSnapshot` & `DebrisSnapshot` (Single Source of Truth)
- **Choice**: Extract reusable `capture_single_ship`, `capture_single_debris`, `spawn_ship_snapshot`, and `spawn_debris_snapshot` helpers in `core/src/save/battle/lib.rs` so both `WorldSave` and `world/chunk` use the exact same snapshot capture/restore logic.
- **Note on cross-ship `Entity` targets**: When an enemy ship is offloaded, its `BrainSave` stores target ship `Uuid`s. When reloaded, target `Uuid`s are resolved against currently live ships (`HashMap<Uuid, Entity>`); any target not currently live resolves cleanly to `None` (and the AI re-evaluates via `TeamIntel` / `Search`).
- **Rationale**: Eliminates duplicate serialization code (`AGENTS.md` single source of truth) and guarantees full fidelity (block HP, battery/turret charge, ammo, barrel angle, `Doctrine`, `Skill`, `Brain`, debris remaining lifetime).

### 4. `BattleOutcome` & `BattleSave` Coordination
- **Choice**:
  - `check_battle_outcome` (`core/src/mission/mod.rs`) reads `Option<Res<ChunkStoreResource>>` and computes `remaining_enemies = live_enemy_ships + chunk_store.total_enemy_count()`. Victory triggers only when `enemy_destroyed && remaining_enemies == 0`.
  - On fresh battle start (`setup_battle` when not loading a save), `ChunkStore::clear()` wipes stale chunk records.
  - On `SaveWorldEvent`, `ChunkStore::flush()` checkpoints/flushes dirty chunks alongside writing `battle.ron`.
  - On `clear_battle_save()` (campaign victory or reset), `battle_chunks.sqlite` (and its `-wal`/`-shm` sidecars) are removed alongside `battle.ron`.
  - On `ChunkStore` schema version mismatch (`PRAGMA user_version != CHUNK_SCHEMA_VERSION`), fail loudly with an explicit error (no silent fallback).

### 5. Hot-Reloadable Chunk Config (`core/assets/config/base/chunk.ron`)
- **Choice**: Load `ChunkConfig { chunk_size, load_radius, despawn_radius, stream_interval_secs }` via `RonAssetLoader` with validation: `chunk_size > 0.0`, `load_radius > 0.0`, `despawn_radius > load_radius`, `load_radius < chunk_size * 0.5` (guarantees 3×3 neighborhood always covers `load_radius`), and `stream_interval_secs > 0.0`.

### 6. Module Layout (`AGENTS.md` Compliant)
```
core/src/world/
├── mod.rs              # WorldPlugin
└── chunk/
    ├── mod.rs          # ChunkPlugin, ChunkStoreResource
    ├── component.rs    # ChunkCoord, ChunkData, ChunkStoreError
    ├── config.rs       # ChunkConfig / ChunkConfigValue + validation
    ├── store.rs        # ChunkStore trait + InMemoryChunkStore + codec (postcard + lz4)
    ├── sqlite.rs       # SqliteChunkStore (rusqlite bundled, WAL, schema version)
    └── system.rs       # stream_chunks, cleanup hooks + unit tests
```

## Risks / Trade-offs

- **Despawned Enemy & `ShipDestroyed` Separation**: Despawning an enemy for chunk offloading MUST NOT emit `ShipDestroyed` or trigger `Death` / loot drops. → Offloading directly despawns the ship entity (and child blocks) without inserting `Death` or writing `ShipDestroyed`.
- **SQLite Build Time on Windows**: `rusqlite` with `bundled` compiles `sqlite3.c` once per target profile. → Cargo caches the compiled artifact across incremental builds; `InMemoryChunkStore` keeps unit tests instantaneous.
- **Unloaded Debris Lifetime**: Detached debris frozen in a chunk preserves its `remaining_secs` without ticking down while unloaded. → Intentional and deterministic; avoids background timers for offloaded chunks.
- **Chunk Save Sync on Crash**: Because `SqliteChunkStore` commits chunk writes during gameplay while `battle.ron` updates on `SaveWorldEvent` / auto-save, a crash between auto-saves could leave `battle_chunks.sqlite` slightly ahead of `battle.ron`. → Mitigate by keeping chunk mutations in an in-memory dirty cache during gameplay and committing them atomically to SQLite on `SaveWorldEvent` (or staging a working copy and committing on save) if strict rollback-to-last-save is required; for active streaming with bounded RAM, flushing to SQLite on chunk eviction + checkpointing on `SaveWorldEvent` is fast and safe, with a dedicated save snapshot copy if strict manual-save rollback is later desired.
