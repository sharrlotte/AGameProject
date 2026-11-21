## MODIFIED Requirements

### Requirement: Offload is save-first and chunk store failures are reported loudly
The streaming system SHALL persist or buffer a live entity's snapshot into `ChunkCache` / `WorldStore` before despawning it, and SHALL despawn an entity only when offload succeeds. When restoring stored entities within `LOAD_RADIUS`, `stream_chunks` SHALL validate and apply snapshots via `apply_bodies` (including live ship `Uuid -> Entity` mappings in `ship_map` so restored enemies retain AI targets on live ships) **before** removing those entities from `ChunkCache` or persisting the updated chunk. Any failed chunk load, save, validation, or restore operation SHALL emit a `ChunkStreamError` message carrying the `ChunkCoord` and a detail string, handled by `on_chunk_stream_error` chained `.after(stream_chunks)`.

#### Scenario: Failed save keeps the live entity and reports the error
- **WHEN** flushing an offloaded chunk to `WorldStore` fails
- **THEN** the error is reported via `ChunkStreamError` and no entity data is silently discarded

#### Scenario: Failed restore preserves stored chunk entities
- **WHEN** `apply_bodies` or snapshot validation fails while loading a stored enemy from chunk `(1, 0)`
- **THEN** the stored snapshot is NOT deleted from `ChunkCache` / `WorldStore`, a `ChunkStreamError` is emitted and logged on the same tick, and no corrupt entity is spawned

#### Scenario: Restored enemy retains AI target pointing at a live player ship
- **WHEN** an enemy ship targeting a live player ship is offloaded and later reloaded into ECS while that player ship is still alive
- **THEN** `apply_bodies` resolves the player ship's `Uuid` from the live ship map and restores the enemy's `Tactic` target instead of degrading to `Tactic::Search`
