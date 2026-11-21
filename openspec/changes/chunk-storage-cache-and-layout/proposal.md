## Why

Chunk streaming currently executes 9 synchronous SQLite `SELECT` queries every `0.25s` (= **36 DB queries/sec on the main thread** for a single stationary player ship, even in empty space) with zero RAM caching, issues individual auto-commit writes per chunk instead of batching, and deletes a chunk's row the moment its last enemy/debris is loaded back into ECS (`sqlite.rs:343-345`), erasing `chunks.visited = 1`. In addition:
- Storing all chunk data in one monolithic `data BLOB` forces decoding and re-compressing every ship and debris whenever future subsystems (POIs, stations, asteroid fields, faction state) read or write their own chunk data.
- `load_in_range` (`world/chunk/system.rs:254-295`) deletes extracted ships from SQLite **before** `apply_bodies` succeeds, risking permanent entity loss on restore error; `apply_bodies` builds `ship_map` only from the restored chunk batch (dropping AI targets pointing to live ships); `on_chunk_stream_error` is not chained `.after(stream_chunks)`.
- `stream_chunks` writes to `chunks` independently of `SaveWorldEvent`, allowing crash divergence between `active_state` and `chunks`.

## What Changes

- Keep `quests` (active/completed player journal) in its own global SQLite table loaded once on slot open (never queried per chunk).
- **BREAKING** Upgrade the `chunks` table to a **single-table, multi-column extensible schema** (`x, y, visited, bodies_blob` — nullable `BLOB` per subsystem so future columns like `pois_blob` or `structures_blob` load in the same single primary-key B-tree lookup while allowing partial-column `UPDATE`s without re-encoding untouched blobs) and bump `WORLD_DB_SCHEMA_VERSION`.
- Stop deleting cleared `visited = 1` chunk rows: set `bodies_blob = NULL` when all enemies/debris leave a chunk so `visited` remains durable across saves and reloads (enabling one-time chunk discovery and encounter prevention).
- Introduce an in-memory **`ChunkCache` resource** holding the active neighborhood's loaded chunks (including negative cache entries for unvisited/empty chunks) and dirty flags:
  - SQLite is queried **only on chunk boundary crossing** (0 DB queries/sec while staying within cached chunks).
  - Dirty chunks flush in batched transactions on eviction and together with `SaveWorldEvent` (routing through `IoWorker` when `async-save-io` is active, keeping `active_state` and `chunks` atomically consistent on save).
- Fix `load_in_range` commit order (mark cache dirty / commit removal **only after** `apply_bodies` succeeds), merge live ship `Uuid -> Entity` mappings into `ship_map` on chunk restore so restored enemies retain targets on live ships, and chain `on_chunk_stream_error.after(stream_chunks)`.

## Capabilities

### New Capabilities
- `chunk-cache`: In-memory spatial `ChunkCache` with negative caching, boundary-triggered loading, dirty-chunk batching, and atomic flush alongside world saves (eliminating per-tick SQLite polling).

### Modified Capabilities
- `chunk-storage`: Upgrade `chunks` to a multi-column per-subsystem blob schema (`x, y, visited, bodies_blob` nullable), preserve durable `visited = 1` rows when `bodies_blob` is `NULL`, and support partial-column updates + batched chunk operations.
- `chunk-streaming`: Operate `stream_chunks` against `ChunkCache` instead of direct per-tick SQLite queries, commit extraction only after `apply_bodies` succeeds, include live ship UUIDs in `ship_map` during chunk restore, and chain `on_chunk_stream_error.after(stream_chunks)`.
- `world-save-slots`: Document atomic flush of dirty cached chunks alongside `world_meta`, `active_state`, and `quests` in `save_world` and updated schema version.

## Non-Goals

- Merging the global `quests` journal table into `chunks` (active quests must remain tracked globally regardless of player chunk).
- Adding specific POI/station/asteroid content schemas now (this change establishes the extensible multi-column pattern and `ChunkCache` boundary).

## Impact

- `core/src/world/chunk/` (`mod.rs`, `component.rs`, `system.rs`, new `cache.rs`).
- `core/src/save/world/` (`component.rs`, `store.rs`, `sqlite.rs`, `system.rs`, `snapshot/apply.rs`).
- `docs/save.md` and `docs/architecture.md` (spec-only updates).
