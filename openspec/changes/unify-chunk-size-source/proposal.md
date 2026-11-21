## Why

Ship/world chunk streaming uses the compile-time constant `CHUNK_SIZE = 50_000.0` (`core/src/constant.rs:32`) and typed `ChunkCoord(IVec2)`, whereas procedural quest generation defines a separate, unvalidated `chunk_size: 50000.0` field in `QuestGenConfig` (`core/assets/config/quest/generation.ron:5`, `core/src/quest/component.rs:134`) and passes raw `IVec2` through `SectorDiscovered` and `QuestId::Chunk(IVec2)`. Having two disconnected definitions violates Single Source of Truth: changing `generation.ron` silently misaligns quest offer/waypoint centers from world chunk coordinates. Additionally, `SectorDiscovered` has no production emitter linking player chunk movement to quest discovery.

## What Changes

- **BREAKING** Remove `chunk_size` from `QuestGenConfig` (`core/src/quest/component.rs`, `core/src/quest/def.rs`) and `core/assets/config/quest/generation.ron`.
- Add `ChunkCoord::center(self, chunk_size: f32) -> Vec2` and `ChunkCoord::from_world_default(pos: Vec2) -> Self` helpers in `core/src/world/chunk/component.rs` using `constant::CHUNK_SIZE` as the single source of truth.
- Standardize `SectorDiscovered { position: Vec2, chunk: ChunkCoord }` (`core/src/world/event.rs`) and `QuestId::Chunk(ChunkCoord)` (`core/src/quest/component.rs`) on `ChunkCoord` instead of raw `IVec2`.
- Update `quest::generator::build` (`core/src/quest/generator.rs`) to compute chunk centers via `chunk.center(CHUNK_SIZE)` instead of `config.chunk_size`.
- Wire a `SectorDiscovered` emitter in `core/src/world/chunk/` when a live player-team ship enters a chunk coordinate not yet discovered in the session/chunk store.

## Capabilities

### New Capabilities
*(none)*

### Modified Capabilities
- `chunk-streaming`: Add `ChunkCoord::center` helper and require chunk tracking to emit `SectorDiscovered { position, chunk: ChunkCoord }` using `CHUNK_SIZE` when a live player ship enters a newly visited chunk.
- `open-world-quests`: Use `ChunkCoord` and `constant::CHUNK_SIZE` as the sole chunk coordinate and size source for `QuestId::Chunk`, `SectorDiscovered`, and generated quest world positions, removing `QuestGenConfig.chunk_size`.

## Non-Goals

- Changing the numeric value of `CHUNK_SIZE` (`50_000.0`) or making chunk streaming sizes RON-configurable.
- Changing quest generation RNG determinism or spacing rules (handled by `deterministic-world-gen`).
- Adding the in-memory `ChunkCache` or multi-column `chunks` table (handled by `chunk-storage-cache-and-layout`).

## Impact

- `core/src/world/chunk/component.rs`, `core/src/world/chunk/system.rs`, `core/src/world/event.rs`.
- `core/src/quest/component.rs`, `core/src/quest/def.rs`, `core/src/quest/generator.rs`, `core/src/quest/system.rs`.
- `core/assets/config/quest/generation.ron`.
- `docs/quests.md` and `docs/architecture.md` (spec-only updates).
