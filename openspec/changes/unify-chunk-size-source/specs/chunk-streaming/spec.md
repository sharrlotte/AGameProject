## MODIFIED Requirements

### Requirement: World space is partitioned into 50,000 x 50,000 storage chunks
The chunk streaming system SHALL map any 2D world position to a `ChunkCoord(IVec2)` using floor division by the compile-time constant `CHUNK_SIZE = 50_000.0`. `ChunkCoord` SHALL be the single canonical chunk coordinate type across streaming, discovery events, and chunk-scoped quests, and SHALL expose `from_world`, `center`, and `neighborhood_3x3`. When a live player-team ship occupies a `ChunkCoord` not yet marked discovered, chunk tracking SHALL emit `SectorDiscovered { position, chunk }` in deterministic `(x, y)` order.

#### Scenario: Positive and negative coordinates map to consistent chunk coordinates
- **WHEN** `ChunkCoord::from_world` is called with `(-1.0, 0.0)` and `(0.0, 0.0)` at `chunk_size = 50_000.0`
- **THEN** `(-1.0, 0.0)` maps to `ChunkCoord(IVec2::new(-1, 0))` and `(0.0, 0.0)` maps to `ChunkCoord(IVec2::new(0, 0))`

#### Scenario: Chunk center matches floor-division bounds
- **WHEN** `ChunkCoord::new(0, 0).center(CHUNK_SIZE)` and `ChunkCoord::new(-1, -1).center(CHUNK_SIZE)` are evaluated
- **THEN** they return `(25_000.0, 25_000.0)` and `(-25_000.0, -25_000.0)` respectively

#### Scenario: Entering a new chunk emits SectorDiscovered once
- **WHEN** a live player-team ship moves into an undiscovered `ChunkCoord(1, 0)`
- **THEN** one `SectorDiscovered` message carrying `ChunkCoord(1, 0)` is emitted and staying inside `(1, 0)` emits no duplicates
