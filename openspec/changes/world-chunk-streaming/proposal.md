## Why

As battles and future procedurally generated maps scale to very large worlds, keeping every enemy ship and detached debris entity permanently simulated in Bevy ECS and Rapier degrades performance, and serializing an entire world into a single in-memory `.ron` file will not scale. Partitioning the world into 50,000 × 50,000 storage chunks with distance-based streaming and single-file SQLite persistence keeps active simulation bounded while remaining scalable and extensible for future open-world systems (procedural generation, factions, stations, fog-of-war).

## What Changes

- Introduce a `world/chunk` feature plugin that partitions world space into `50,000 × 50,000` storage chunks (`ChunkCoord(IVec2)`) and streams **enemy ships** (`ShipSnapshot`) and **detached debris** (`DebrisSnapshot`) between live ECS and chunk storage using distance hysteresis (`load_radius < despawn_radius`, loaded from `.ron`) over the player's 3×3 chunk neighborhood.
- Keep player/allied ships always live (never despawned into chunks).
- Define a backend-agnostic `ChunkStore` trait with:
  - `SqliteChunkStore` backed by a single bundled SQLite file (`rusqlite`, WAL mode, `PRAGMA user_version` schema versioning) storing compressed chunk blobs (`postcard` + `lz4_flex`) alongside indexed `enemy_count` metadata.
  - `InMemoryChunkStore` for fast, deterministic unit tests.
- Update `check_battle_outcome` so `BattleOutcome::Victory` counts both live enemy ships and unloaded enemy ships persisted in `ChunkStore` (via O(1)/indexed `SUM(enemy_count)` without deserializing chunk blobs), preventing false Victory when enemies are out of range.
- Integrate `ChunkStore` lifecycle with `save/battle` (`SaveWorldEvent`, `LoadWorldEvent`, `clear_battle_save`, and fresh mission start) and add hot-reloadable chunk streaming config in `.ron` with validation.
- Update `/docs` (`simulation.md`, `save.md`, `config.md`, `architecture.md`) to document chunk streaming and SQLite chunk storage.

## Capabilities

### New Capabilities
- `chunk-streaming`: 50,000 × 50,000 spatial chunk partitioning and distance-hysteresis streaming (`load_radius` / `despawn_radius`) across the player's 3×3 chunk neighborhood for enemy ships and detached debris.
- `chunk-storage`: Single-file SQLite (`rusqlite` bundled, WAL mode) persistence behind a `ChunkStore` trait storing compressed chunk snapshots and indexed metadata (`enemy_count`), plus `InMemoryChunkStore` for tests.

### Modified Capabilities
- `battle-outcome`: Victory evaluation must include unloaded enemies remaining in `ChunkStore` alongside live ECS enemy ships.
- `world-save`: Battle save, load, and clear flows must coordinate with `ChunkStore` so unloaded chunk entities persist across save/load and are cleaned up on battle reset or campaign victory.

## Impact

- **Dependencies**: Add `rusqlite` (with `bundled` feature), `postcard`, and `lz4_flex` to `core/Cargo.toml`.
- **Systems & Modules**:
  - New `core/src/world/chunk/` module (`mod.rs`, `component.rs`, `config.rs`, `store.rs`, `sqlite.rs`, `system.rs`) registered in `core/src/main.rs`.
  - `core/src/mission/mod.rs`: update `check_battle_outcome` to include `ChunkStore::total_enemy_count()`.
  - `core/src/save/battle/`: expose reusable single-ship / single-debris snapshot capture & spawn helpers and coordinate battle save/load/clear with `ChunkStore`.
  - `core/assets/config/base/`: hot-reloadable chunk streaming config (chunk size, load/despawn radii, stream interval).
- **Documentation**: Updates `docs/simulation.md`, `docs/save.md`, `docs/config.md`, and `docs/architecture.md`.
