## 1. Dependencies & Config

- [ ] 1.1 Add `rusqlite` (with `bundled` feature), `postcard`, and `lz4_flex` to `core/Cargo.toml`
- [ ] 1.2 Create `core/assets/config/base/chunk.ron` with `chunk_size` (`50000.0`), `load_radius`, `despawn_radius`, and `stream_interval_secs`
- [ ] 1.3 Implement `ChunkConfig` / `ChunkConfigValue` asset loader, validation (`despawn_radius > load_radius`, `load_radius < chunk_size * 0.5`), and hot-reload bridge in `core/src/world/chunk/config.rs` and wire into `ConfigPlugin`

## 2. Chunk Storage Backend (`ChunkStore` + SQLite)

- [ ] 2.1 Create `core/src/world/chunk/component.rs` with `ChunkCoord`, `ChunkData { enemies, debris }`, `ChunkStoreError`, `CHUNK_SCHEMA_VERSION`, and `neighborhood_3x3`
- [ ] 2.2 Create `core/src/world/chunk/store.rs` defining the `ChunkStore` trait, `postcard` + `lz4_flex` encode/decode helpers, and `InMemoryChunkStore`
- [ ] 2.3 Create `core/src/world/chunk/sqlite.rs` implementing `SqliteChunkStore` (`rusqlite` bundled, WAL mode, `PRAGMA user_version` check, `chunks(x, y, enemy_count, data)` table, indexed `total_enemy_count()`, row deletion on empty chunk)
- [ ] 2.4 Add unit tests for `ChunkCoord`, `InMemoryChunkStore`, and `SqliteChunkStore` (reopen round-trip, unvisited chunk returns `None`, `total_enemy_count()`, empty-row cleanup, schema version mismatch error)

## 3. Reusable Snapshot Helpers & Chunk Streaming

- [ ] 3.1 Extract reusable single-ship and single-debris capture/spawn helpers in `core/src/save/battle/lib.rs` so both `WorldSave` and `world/chunk` share a single serialization path
- [ ] 3.2 Create `core/src/world/chunk/system.rs` and `core/src/world/chunk/mod.rs` (`ChunkPlugin`, `ChunkStoreResource`) implementing periodic distance-hysteresis streaming across the 3×3 neighborhood of live player-team ships for enemy ships and detached debris (never despawning player-team ships, never emitting `ShipDestroyed` on offload)
- [ ] 3.3 Register `WorldPlugin` / `ChunkPlugin` in `core/src/main.rs` and add unit tests verifying offload beyond `despawn_radius`, reload within `load_radius` (including across a 50k chunk border), hysteresis deadband, debris lifetime preservation, and player-ship immunity

## 4. Battle Outcome & Battle Save Integration

- [ ] 4.1 Update `check_battle_outcome` in `core/src/mission/mod.rs` to add `ChunkStore::total_enemy_count()` to live enemy ships and add a unit test confirming unloaded enemies prevent premature Victory
- [ ] 4.2 Integrate `ChunkStore` with `core/src/save/battle/` and `core/src/mission/mod.rs`: clear chunks on fresh non-loaded mission start, flush `battle_chunks.sqlite` on `SaveWorldEvent`, keep store attached on `LoadWorldEvent`, and remove `battle_chunks.sqlite` (plus WAL/SHM sidecars) in `clear_battle_save`
- [ ] 4.3 Add save/load integration test verifying a battle with offloaded chunk enemies and debris survives save/load and clears cleanly

## 5. Documentation & Verification

- [ ] 5.1 Update `/docs/simulation.md`, `/docs/save.md`, `/docs/config.md`, and `/docs/architecture.md` with chunk streaming and SQLite storage specs
- [ ] 5.2 Run targeted unit tests (`cargo test`) and verify in-game with `cargo dev`
