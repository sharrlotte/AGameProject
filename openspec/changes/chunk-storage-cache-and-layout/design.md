## Context

Audit of `core/src/world/chunk/system.rs:21-296` and `core/src/save/world/sqlite.rs:324-365, 510-517` identified why chunk persistence will become a bottleneck as more world content is added:
1. **36 un-cached SQLite `SELECT`s/sec**: Every `0.25s`, `load_in_range` iterates all 9 `neighbor_chunks` and calls `store.load_chunk(coord)` synchronously, re-preparing the SQL statement each time even when the player has not moved and all 9 chunks are empty.
2. **Monolithic blob & lost `visited` state**: `chunks(x, y, visited, enemy_count, data)` packs everything into one `data` blob and deletes the entire row (`remove_chunk`) whenever `enemies.is_empty() && debris.is_empty()`, erasing `visited = 1` and making cleared chunks indistinguishable from unvisited chunks.
3. **Data loss & consistency hazards**:
   - `load_in_range` calls `store.save_chunk` (deleting entities from SQLite) *before* `apply_bodies` runs — any restore error permanently drops those entities.
   - `stream_chunks` commits to `chunks` every `0.25s` independently of `SaveWorldEvent` (`30s`), so a crash can duplicate or lose entities across `active_state` vs `chunks`.
   - Restored chunk enemies lose AI `Tactic` targets pointing at live player/ally ships because `apply_bodies` only populates `ship_map` with newly spawned chunk ships.

## Goals / Non-Goals

**Goals:**
- Reduce idle chunk SQLite queries from 36/sec to **0/sec** via an in-memory `ChunkCache` with negative caching.
- Keep **1 primary-key lookup per chunk load** as new chunk-scoped features (POIs, stations, asteroids, resources) are added, while allowing each subsystem to update its own nullable `BLOB` column independently without re-encoding untouched columns.
- Keep `quests` (active/completed player journal) in its own global table loaded once at session start.
- Make `chunks.visited` durable across save/load (never delete a visited chunk row just because `bodies_blob` became empty).
- Ensure save-first / restore-before-commit safety, atomic dirty-chunk flush with `save_world`, and full live-ship `ship_map` resolution on chunk reload.

**Non-Goals:**
- Defining concrete POI/station/asteroid game features (future proposals simply add nullable `*_blob` columns to `chunks` and fields to `CachedChunk`).
- Building cross-chunk secondary SQL indexes unless a future feature requires galaxy-wide SQL filtering.

## Decisions

### 1. Single `chunks` Table with Per-Subsystem Nullable Blob Columns (Separate from `quests`)
- **Choice**:
  ```sql
  CREATE TABLE IF NOT EXISTS chunks (
      x           INTEGER NOT NULL,
      y           INTEGER NOT NULL,
      visited     INTEGER NOT NULL DEFAULT 0,
      bodies_blob BLOB,
      PRIMARY KEY (x, y)
  );
  ```
  - `quests` stays its own global table (`quest_id`, `status`, `record_blob`), loaded once on slot open.
  - `chunks` stores one row per visited or populated `(x, y)` coordinate. Unused `enemy_count` column is removed.
  - When all enemies and debris leave a chunk, `bodies_blob` becomes `NULL` while `visited = 1` remains intact (unvisited chunks still have no row at all).
  - Future chunk subsystems (e.g., `pois_blob`, `structures_blob`, `asteroids_blob`) add one nullable `BLOB` column to `chunks`: loading a chunk fetches all columns in **one primary-key B-tree lookup**, while writing dirty bodies updates only `bodies_blob` (`UPDATE chunks SET visited = ?3, bodies_blob = ?4 WHERE x = ?1 AND y = ?2` / upsert) without re-compressing untouched columns.
- **Alternatives Considered**:
  - *Single monolithic `data BLOB`*: Forces re-encoding all POIs/stations every time a stray enemy drifts across a chunk border.
  - *Separate chunk tables per subsystem (`chunk_bodies`, `chunk_pois`, …)*: Requires $K$ separate B-tree lookups per chunk transition; only worthwhile if cross-chunk SQL queries (`WHERE faction_id = ?`) are required later.

### 2. In-Memory `ChunkCache` + Boundary-Triggered Loading + Atomic Flush
- **Choice**: Add `#[derive(Resource, Default)] pub struct ChunkCache` in `core/src/world/chunk/cache.rs`:
  ```rust
  pub struct CachedChunk {
      pub visited: bool,
      pub bodies: Option<ChunkBodies>, // enemies + debris
      pub dirty_meta: bool,
      pub dirty_bodies: bool,
  }

  #[derive(Resource, Default)]
  pub struct ChunkCache {
      entries: HashMap<ChunkCoord, CachedChunk>, // Some(empty) acts as negative cache!
  }
  ```
  - **Load on miss only**: `stream_chunks` computes the union of active 3×3 player neighborhoods (`active_set`). Only coordinates NOT already in `ChunkCache` are fetched in one batched `WorldStore::load_chunks(&[ChunkCoord])` call (using `prepare_cached` in SQLite). Unvisited chunks insert a clean `CachedChunk { visited: false, bodies: None, .. }` negative-cache entry so subsequent ticks do **0 DB queries**.
  - **Offload & load-in-range in RAM**: Offloading appends snapshots to `cache[coord].bodies` and sets `dirty_bodies = true`; loading into ECS extracts in-range snapshots from `cache[coord].bodies`, calls `apply_bodies` **first**, and only mutates `cache[coord]` + sets `dirty_bodies = true` if `apply_bodies` succeeds!
  - **Eviction & atomic world save**:
    - Chunks outside the 5×5 keep-ring (1-chunk eviction buffer beyond the 3×3 active ring to avoid border thrashing) flush dirty columns via `save_chunks_batch` and evict from RAM.
    - When `SaveWorldEvent` runs (`save_world`), `ChunkCache::drain_dirty()` writes all dirty cached chunks in the **same `BEGIN IMMEDIATE` transaction** as `world_meta`, `active_state`, and `quests` — eliminating crash inconsistency between `active_state` and `chunks`!
    - On slot create/load/exit, `ChunkCache::clear()` resets RAM state cleanly.

### 3. Live `ship_map` Resolution & Error Handler Chaining
- **Choice**: Pass an existing `live_ships: &HashMap<Uuid, Entity>` (collected from live `Query<(Entity, &Ship)>` before spawning) into `apply_bodies` so restored chunk enemies keep valid `Brain` targets pointing at live player or allied ships. Chain `on_chunk_stream_error.after(stream_chunks)` in `WorldChunkPlugin`.

## Risks / Trade-offs

- **RAM footprint of `ChunkCache`** → With a 5×5 keep-ring around player ships, at most ~25 chunks reside in RAM per ship (each empty negative-cache entry is <64 bytes), bounding memory tightly while guaranteeing zero thrashing across chunk borders.
- **Unflushed dirty chunks on abrupt crash** → Dirty chunks outside the 5×5 ring flush on eviction, and in-ring dirty chunks flush atomically with `active_state` on every autosave/exit save — making crashes strictly consistent instead of half-updated!
- **Schema bump** → Replacing `data` + `enemy_count` with nullable `bodies_blob` increments `WORLD_DB_SCHEMA_VERSION` (cleanly batched if multiple schema-bumping proposals land together).
