## 1. Extend `ChunkCoord` and Remove Duplicate `chunk_size`

- [ ] 1.1 Add `#[serde(transparent)]` (or verify wire compatibility) and `pub fn center(self, chunk_size: f32) -> Vec2` to `ChunkCoord` in `core/src/world/chunk/component.rs`.
- [ ] 1.2 Remove `chunk_size` from `QuestGenConfig` (`core/src/quest/component.rs`), `QuestGenConfig::validate` (`core/src/quest/def.rs`), and `core/assets/config/quest/generation.ron`.
- [ ] 1.3 Update `SectorDiscovered.chunk` (`core/src/world/event.rs`) and `QuestId::Chunk` (`core/src/quest/component.rs`) to store `ChunkCoord` instead of raw `IVec2`.
- [ ] 1.4 Update `quest::generator::build` (`core/src/quest/generator.rs`) to compute `let center = chunk.center(CHUNK_SIZE);` using `crate::constant::CHUNK_SIZE`.

## 2. Emit `SectorDiscovered` from Player Chunk Movement

- [ ] 2.1 Add a `DiscoveredChunks` resource in `core/src/world/chunk/component.rs` (reset on slot switch/exit) and emit sorted `SectorDiscovered` messages in `core/src/world/chunk/system.rs` whenever a live player-team ship enters an unvisited `ChunkCoord`.

## 3. Verification and Docs

- [ ] 3.1 Add unit tests verifying `ChunkCoord::center` for positive/negative coordinates, alignment with `ChunkCoord::from_world`, and single deterministic emission of `SectorDiscovered`.
- [ ] 3.2 Update `docs/quests.md` and `docs/architecture.md` (spec only, no notes) to record `CHUNK_SIZE` and `ChunkCoord` as the single chunk-size source.
