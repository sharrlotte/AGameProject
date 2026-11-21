## ADDED Requirements

### Requirement: ChunkCache eliminates per-tick SQLite polling with negative caching and eviction hysteresis
The chunk subsystem SHALL maintain an in-memory `ChunkCache` resource caching loaded chunk data and negative-cache entries for unvisited/empty chunks within a 5×5 keep-ring around live player-team ships. `stream_chunks` SHALL query `WorldStore` only on cache misses (chunk boundary crossings) and SHALL perform zero SQLite reads while players remain inside cached chunks. Chunks leaving the 5×5 keep-ring SHALL flush dirty columns in a batch before eviction; `ChunkCache` SHALL clear on slot switch or session exit.

#### Scenario: Stationary player performs zero repeated SQLite chunk queries
- **WHEN** a player ship stays inside chunk `(0, 0)` across 20 `StreamingTimer` ticks
- **THEN** `WorldStore` loads the 3×3 neighborhood once on the initial cache miss and executes zero chunk `SELECT` queries on the remaining 19 ticks

#### Scenario: Evicting a dirty distant chunk flushes it before removal from RAM
- **WHEN** a player flies beyond the 5×5 keep-ring of a chunk containing offloaded enemies
- **THEN** the dirty chunk is persisted to `WorldStore` before being evicted from `ChunkCache`, and returning later reloads those enemies accurately

### Requirement: Dirty cached chunks flush atomically with world saves
When `SaveWorldEvent` commits a save slot, all dirty entries in `ChunkCache` SHALL be written to the `chunks` table inside the same `BEGIN IMMEDIATE` transaction as `world_meta`, `active_state`, and `quests`, preventing active/chunk entity duplication or loss on crash.

#### Scenario: World save commits dirty chunks and active_state together
- **WHEN** an enemy is offloaded into `ChunkCache` and `SaveWorldEvent` commits
- **THEN** `active_state` (without the offloaded enemy) and `chunks` (with the offloaded enemy) commit atomically in one transaction and the chunk's dirty flag clears
