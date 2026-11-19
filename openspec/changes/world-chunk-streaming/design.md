## Context

With `world-save-slots` (single-file SQLite per save slot via `WorldStore`) and `open-world-quests` (open-world gameplay with origin respawn and no arena Victory screen) in place, the active save slot already owns a `chunks` table inside `<data_dir>/saves/<slot_id>.db`. However, enemy ships and detached debris currently remain permanently simulated in Bevy ECS and Rapier regardless of distance from the player.

## Goals / Non-Goals

**Goals:**
- Partition world space into `50,000 × 50,000` storage chunks (`ChunkCoord(IVec2)`), cleanly separating *spatial storage partitioning* from *active simulation distance*.
- Stream **enemy ships** (`ShipSnapshot`) and **detached debris** (`DebrisSnapshot`) between live ECS and the active `WorldStore`'s `chunks` table using distance hysteresis (`load_radius < despawn_radius`, loaded from `.ron`) across the 3×3 chunk neighborhood of live player-team ships.
- Keep player/allied ships always live in ECS.
- Persist chunks inside the exact same single-file save slot database (`saves/<slot_id>.db`) via `WorldStore` (`SqliteWorldStore` in production, `InMemoryWorldStore` in `QuickBattle` and unit tests).

**Non-Goals:**
- Procedural world/encounter generation itself (unvisited chunks return `Ok(None)` from `WorldStore::load_chunk`, establishing the exact hook for future procedural generators).
- Off-screen background simulation of unloaded enemies (unloaded enemies and debris freeze at their saved snapshot state until reloaded).
- Streaming player-team/allied ships or transient projectiles/drops.

## Decisions

### 1. Spatial Storage (50k × 50k Chunks) + Distance Hysteresis (`load_radius` / `despawn_radius`)
- **Choice**:
  - `ChunkCoord::from_world(pos, chunk_size)` maps world coordinates via `(pos / chunk_size).floor().as_ivec2()` with default `chunk_size = 50_000.0`.
  - A periodic streaming system (throttled by `ChunkConfig.stream_interval_secs`, e.g. `0.25s`, running in `FixedUpdate` `SimulationSet::Cleanup` while in `SceneState::Game`):
    1. **Offload (Despawn & Save)**: Any live non-player `Ship` or `Debris` farther than `despawn_radius` (e.g. `10_000.0`) from the nearest live player-team ship is captured as `ShipSnapshot` / `DebrisSnapshot` (reusing `save/world/snapshot.rs` helpers), saved into its `ChunkCoord` in `WorldStore`, and despawned from ECS without emitting `ShipDestroyed` or spawning loot.
    2. **Load (Spawn)**: For each `ChunkCoord` in the 3×3 neighborhood of each live player-team ship, inspect stored chunk entries; any `ShipSnapshot` or `DebrisSnapshot` within `load_radius` (e.g. `8_000.0`) of any live player-team ship is removed from the chunk record, saved back (or deleted if empty), and spawned into live ECS with full snapshot fidelity.
- **Rationale**: Zero chunk-border pop-in even right on a `50,000` boundary, bounded live Rapier/AI entity count, hysteresis prevents spawn/despawn thrashing, and multi-ally distance checks keep enemies near allied scouts live.

### 2. Unified Single-File `WorldStore` Integration (`chunks` Table)
- **Choice**: Extend `WorldStore` (`core/src/save/world/store.rs` and `sqlite.rs`) with:
  ```rust
  fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<ChunkData>, WorldStoreError>;
  fn save_chunk(&mut self, coord: ChunkCoord, data: &ChunkData) -> Result<(), WorldStoreError>;
  fn remove_chunk(&mut self, coord: ChunkCoord) -> Result<(), WorldStoreError>;
  ```
  - Uses the `chunks(x, y, visited, enemy_count, data)` table already created inside `saves/<slot_id>.db` by `world-save-slots`, encoding `ChunkData { enemies, debris }` with the shared `postcard` + `lz4_flex` codec.
  - Empty chunks (no enemies and no debris remaining) delete their row (or mark `visited = 1` once procedural generation is added) so empty unvisited space costs 0 bytes.
  - `QuickBattle` uses `InMemoryWorldStore`, so chunk streaming works identically in sandbox mode with zero disk writes.
- **Rationale**: Zero extra files per save slot, atomic consistency with `world_meta`, `active_state`, `designs`, and `quests`, and ready for future procedural generation / faction / sector queries.

### 3. Hot-Reloadable Chunk Config (`core/assets/config/base/chunk.ron`)
- **Choice**: Load `ChunkConfig { chunk_size, load_radius, despawn_radius, stream_interval_secs }` via `RonAssetLoader` with validation: `chunk_size > 0.0`, `load_radius > 0.0`, `despawn_radius > load_radius`, `load_radius < chunk_size * 0.5`, and `stream_interval_secs > 0.0`.

### 4. Module Layout (`AGENTS.md` Compliant)
```
core/src/world/
├── mod.rs              # WorldPlugin
└── chunk/
    ├── mod.rs          # ChunkPlugin
    ├── component.rs    # ChunkCoord, ChunkData, neighborhood_3x3
    ├── config.rs       # ChunkConfig / ChunkConfigValue + validation
    └── system.rs       # stream_chunks + unit tests
```

## Risks / Trade-offs

- **Despawned Enemy & `ShipDestroyed` Separation**: Offloading an enemy MUST NOT emit `ShipDestroyed` (which would falsely advance `DestroyEnemies` quests) or spawn loot. → Offloading despawns directly without inserting `Death` or writing `ShipDestroyed`.
- **Cross-Ship AI Target `Uuid`s on Reload**: Offloaded enemy `BrainSave` target `Uuid`s resolve against currently live ships on reload; any target not live resolves cleanly to `None` and AI re-evaluates naturally via `TeamIntel` / `Search`.
- **Unloaded Debris Lifetime**: Detached debris frozen in a chunk preserves `remaining_secs` without ticking down while unloaded. → Deterministic and zero background CPU cost.
