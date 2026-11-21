## 1. Upgrade `chunks` Schema and `WorldStore` Batch/Column API

- [ ] 1.1 Update `CREATE_SCHEMA_SQL` in `core/src/save/world/sqlite.rs` so `chunks` uses `(x, y, visited, bodies_blob)` (nullable `bodies_blob`, removing unused `enemy_count`), and bump `WORLD_DB_SCHEMA_VERSION`.
- [ ] 1.2 Define `ChunkBodies` (enemies + debris) and `ChunkEntry { visited: bool, bodies: Option<ChunkBodies> }` in `core/src/world/chunk/component.rs`.
- [ ] 1.3 Add `load_chunks`, `save_chunk_entry` (partial column upsert preserving `visited = 1` when `bodies_blob` is `NULL`), and `save_chunks_batch` (single transaction, using `prepare_cached`) to `WorldStore`, `SqliteWorldStore`, and `InMemoryWorldStore`.
- [ ] 1.4 Extend `WorldStore::save_world` to accept `&[(ChunkCoord, ChunkEntry)]` dirty chunks and commit them in the same `BEGIN IMMEDIATE` transaction as `world_meta`, `active_state`, and `quests`.

## 2. Implement In-Memory `ChunkCache`

- [ ] 2.1 Create `core/src/world/chunk/cache.rs` implementing `ChunkCache` resource with `CachedChunk { visited, bodies, dirty_meta, dirty_bodies }`, negative caching for unvisited chunks, 5×5 keep-ring eviction detection, and `drain_dirty()` / `clear()`.
- [ ] 2.2 Register `ChunkCache` in `WorldChunkPlugin` (`core/src/world/chunk/mod.rs`) and clear it on slot create/load/Home/Skirmish transitions (`core/src/save/world/system.rs`).

## 3. Migrate `stream_chunks` to `ChunkCache` and Fix Restore Ordering

- [ ] 3.1 Refactor `stream_chunks` (`core/src/world/chunk/system.rs`) to populate cache misses via `load_chunks`, perform offload/reload against `ChunkCache` in RAM, mark newly visited player chunks (`visited = true`, `dirty_meta = true`), and flush+evict chunks outside the 5×5 keep-ring.
- [ ] 3.2 Reorder `load_in_range` so `apply_bodies` runs **before** mutating `ChunkCache` / committing extraction, preventing entity loss on restore error.
- [ ] 3.3 Update `apply_bodies` (`core/src/save/world/snapshot/apply.rs`) to accept live ship `Uuid -> Entity` mappings so chunk-restored enemies retain AI targets on live ships.
- [ ] 3.4 Chain `on_chunk_stream_error.after(stream_chunks)` in `core/src/world/chunk/mod.rs`, and wire `ChunkCache` dirty flush into `handle_save_world_event` (integrating with `IoWorker` when `async-save-io` is applied).

## 4. Verification and Docs

- [ ] 4.1 Add unit tests covering: zero repeated `WorldStore` queries for stationary players, durable `visited = 1` with `bodies_blob = NULL`, atomic `save_world` + dirty-chunk flush, failed restore preserving cached/stored entities, live-ship AI target resolution on chunk reload, and 5×5 eviction flush.
- [ ] 4.2 Update `docs/save.md` and `docs/architecture.md` (spec only, no notes) documenting the multi-column `chunks` schema, durable `visited`, and `ChunkCache`.
