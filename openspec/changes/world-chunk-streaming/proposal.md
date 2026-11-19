## Why

As the open world scales to large procedurally generated maps, keeping every enemy ship and detached debris entity permanently simulated in Bevy ECS and Rapier degrades performance. Partitioning world space into `50,000 × 50,000` storage chunks with distance-based streaming directly into the active save slot's SQLite `WorldStore` (`chunks` table) keeps active simulation bounded while keeping each save slot a single self-contained `.db` file.

## What Changes

- Introduce a `core/src/world/chunk/` feature plugin that partitions world space into `50,000 × 50,000` storage chunks (`ChunkCoord(IVec2)`) and streams **enemy ships** (`ShipSnapshot`) and **detached debris** (`DebrisSnapshot`) between live ECS and the active `WorldStore`'s `chunks` table using distance hysteresis (`load_radius < despawn_radius`, loaded from `.ron`) across the 3×3 chunk neighborhood of live player-team ships.
- Keep player/allied ships always live (never despawned into chunks).
- Add chunk persistence methods (`load_chunk`, `save_chunk`, `remove_chunk`) to `WorldStore` (`SqliteWorldStore` and `InMemoryWorldStore`), storing `postcard` + `lz4_flex` compressed `ChunkData` blobs alongside indexed metadata (`x, y, visited, enemy_count`) inside the same save-slot `.db` file.
- Add hot-reloadable chunk streaming config (`core/assets/config/base/chunk.ron`) with validation (`despawn_radius > load_radius`, `load_radius < chunk_size * 0.5`).
- Update `/docs/simulation.md`, `/docs/save.md`, `/docs/config.md`, and `/docs/architecture.md`.

## Capabilities

### New Capabilities
- `chunk-streaming`: 50,000 × 50,000 spatial chunk partitioning and distance-hysteresis streaming (`load_radius` / `despawn_radius`) across the 3×3 chunk neighborhood of live player-team ships for enemy ships and detached debris.
- `chunk-storage`: Chunk table persistence (`chunks(x, y, visited, enemy_count, data)`) integrated into `WorldStore` (`SqliteWorldStore` and `InMemoryWorldStore`) with `postcard` + `lz4_flex` compression and zero storage cost for unvisited chunks.

### Modified Capabilities

## Impact

- **Dependencies**: Depends on `world-save-slots` (`WorldStore`, `rusqlite`, `postcard`, `lz4_flex`) and `open-world-quests` (no `BattleOutcome::Victory` coupling).
- **Systems & Modules**:
  - New `core/src/world/chunk/` module (`mod.rs`, `component.rs`, `config.rs`, `system.rs`) registered in `core/src/main.rs`.
  - `core/src/save/world/`: extend `WorldStore` (`SqliteWorldStore` + `InMemoryWorldStore`) with `load_chunk` / `save_chunk` / `remove_chunk` backed by the `chunks` table inside `saves/<slot_id>.db`, and reuse single-ship / single-debris snapshot capture & spawn helpers.
  - `core/assets/config/base/chunk.ron`: hot-reloadable chunk streaming configuration.
- **Documentation**: Updates `docs/simulation.md`, `docs/save.md`, `docs/config.md`, and `docs/architecture.md`.
