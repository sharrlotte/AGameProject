## Context

Chunk streaming (`core/src/constant.rs:32`) and quest generation (`core/assets/config/quest/generation.ron:5`) happen to share the value `50,000.0` today, but they have zero compile-time or runtime link:
- `world/chunk/component.rs:8,70-72` defines `ChunkCoord(pub IVec2)` and `ChunkCoord::from_world(pos, CHUNK_SIZE)`.
- `quest/component.rs:134` defines `QuestGenConfig.chunk_size: f32`, validated only for `> 0.0` (`quest/def.rs:23`).
- `world/event.rs:6` (`SectorDiscovered.chunk`) and `quest/component.rs:10` (`QuestId::Chunk`) use raw `IVec2` rather than `ChunkCoord`.
- No system in `world/chunk/` currently emits `SectorDiscovered`.

## Goals / Non-Goals

**Goals:**
- Establish `constant::CHUNK_SIZE` and `ChunkCoord` as the single source of truth for spatial chunk coordinates and world-space chunk centers across both entity streaming and quests.
- Remove the duplicate `chunk_size` field from `QuestGenConfig` and `generation.ron`.
- Emit `SectorDiscovered` deterministically when player ships enter an unvisited `ChunkCoord`.

**Non-Goals:**
- Introducing a separate macro-sector grid size (macro-grid quest spacing is expressed as an integer multiple `quest_grid_stride` of `CHUNK_SIZE` in `deterministic-world-gen`, preserving one base grid).
- Rewriting chunk SQLite persistence (handled by `chunk-storage-cache-and-layout`).

## Decisions

### 1. `constant::CHUNK_SIZE` + `ChunkCoord` Everywhere
- **Choice**:
  - Delete `QuestGenConfig.chunk_size` and its entry in `generation.ron`.
  - Add `pub fn center(self, chunk_size: f32) -> Vec2 { self.0.as_vec2() * chunk_size + Vec2::splat(chunk_size * 0.5) }` on `ChunkCoord`.
  - Change `SectorDiscovered.chunk` and `QuestId::Chunk` from `IVec2` to `ChunkCoord` (which is a transparent `#[serde(transparent)]` or tuple newtype over `IVec2`, preserving `"chunk:{x}:{y}"` keys and zero-cost `Copy`).
- **Rationale**: Eliminates any possibility of ship streaming and quest generation disagreeing on chunk size or origin math, and enforces type safety so arbitrary grid/pixel `IVec2` values cannot be passed as chunk coordinates.
- **Alternative Considered**: *Cross-validating `QuestGenConfig.chunk_size == CHUNK_SIZE` at startup* — redundant boilerplate for a value that is never allowed to diverge.

### 2. Player Chunk Discovery Emitter in `world/chunk/`
- **Choice**: Track player-visited chunks in a `DiscoveredChunks(HashSet<ChunkCoord>)` resource (or `ChunkCache` once `chunk-storage-cache-and-layout` lands) updated in `world/chunk/system.rs` whenever a live player-team ship occupies a `ChunkCoord` not yet marked visited; sort newly discovered coordinates by `(x, y)` before emitting `SectorDiscovered { position: coord.center(CHUNK_SIZE), chunk: coord }` so multi-ship fleets emit discoveries in deterministic order.
- **Rationale**: Connects `world/chunk/` to `quest/` cleanly through the existing `SectorDiscovered` message (respecting `AGENTS.md`: *"Prefer event for communication"* and keeping `world/` unaware of `quest/`).

## Risks / Trade-offs

- **Serialized `QuestId::Chunk(ChunkCoord)` wire format** → Annotate `ChunkCoord` with `#[serde(transparent)]` (or accept schema cutover) so its `postcard` / RON representation stays identical to `IVec2`.
- **Persistence of `visited` across reloads** → Full durable `visited` persistence across save/load is completed by `chunk-storage-cache-and-layout` (which stops deleting empty visited rows in `chunks`); until that change lands, `DiscoveredChunks` prevents duplicate emissions within a session and `QuestTracker` deduplicates already-active/completed chunk quests across loads.
