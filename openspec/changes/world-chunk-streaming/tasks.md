## 1. Config & Chunk Data Types

- [ ] 1.1 Create `core/assets/config/base/chunk.ron` with `chunk_size` (`50000.0`), `load_radius`, `despawn_radius`, and `stream_interval_secs`
- [ ] 1.2 Create `core/src/world/chunk/component.rs` defining `ChunkCoord`, `ChunkData { enemies, debris }`, and `neighborhood_3x3`
- [ ] 1.3 Create `core/src/world/chunk/config.rs` implementing `ChunkConfig` / `ChunkConfigValue`, validation (`despawn_radius > load_radius`, `load_radius < chunk_size * 0.5`), and hot-reload bridge wired into `ConfigPlugin`

## 2. WorldStore Chunk Methods

- [ ] 2.1 Extend `WorldStore` in `core/src/save/world/store.rs` and `sqlite.rs` with `load_chunk`, `save_chunk`, and `remove_chunk` backed by the `chunks` table (`postcard` + `lz4_flex` payload, `enemy_count` column, row deletion on empty chunk)
- [ ] 2.2 Add unit tests for `ChunkCoord`, `InMemoryWorldStore`, and `SqliteWorldStore` chunk persistence (reopen round-trip, unvisited returns `None`, empty-row cleanup)

## 3. Chunk Streaming Plugin & Verification

- [ ] 3.1 Create `core/src/world/chunk/system.rs`, `core/src/world/chunk/mod.rs` (`ChunkPlugin`), and `core/src/world/mod.rs` (`WorldPlugin`) implementing periodic distance-hysteresis streaming across the 3×3 neighborhood of live player-team ships for enemy ships and detached debris (never despawning player-team ships, never emitting `ShipDestroyed` on offload)
- [ ] 3.2 Register `WorldPlugin` in `core/src/main.rs` and add unit tests verifying offload beyond `despawn_radius`, reload within `load_radius` (including across a 50k chunk border), hysteresis deadband, debris lifetime preservation, and player-ship immunity
- [ ] 3.3 Update `/docs/simulation.md`, `/docs/save.md`, `/docs/config.md`, and `/docs/architecture.md` with chunk streaming specifications
- [ ] 3.4 Run targeted unit tests (`cargo test`) and verify in-game with `cargo dev`
