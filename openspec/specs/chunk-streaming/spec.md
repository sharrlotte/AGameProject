# chunk-streaming Specification

## Purpose
Defines spatial partitioning and distance-hysteresis streaming requirements for offloading distant ships and debris to chunk storage and loading them back into live ECS.
## Requirements
### Requirement: World space is partitioned into 50,000 x 50,000 storage chunks
The chunk streaming system SHALL map any 2D world position to a `ChunkCoord(IVec2)` using floor division by the compile-time constant `CHUNK_SIZE = 50_000.0`. Each `ChunkCoord` SHALL support computing its 3×3 neighborhood (itself and 8 adjacent chunks).

#### Scenario: Positive and negative coordinates map to consistent chunk coordinates
- **WHEN** `ChunkCoord::from_world` is called with `(-1.0, 0.0)` and `(0.0, 0.0)` at `chunk_size = 50_000.0`
- **THEN** `(-1.0, 0.0)` maps to `ChunkCoord(IVec2::new(-1, 0))` and `(0.0, 0.0)` maps to `ChunkCoord(IVec2::new(0, 0))`

#### Scenario: 3x3 neighborhood includes all 9 surrounding chunks
- **WHEN** `neighborhood_3x3` is called on `ChunkCoord(IVec2::new(2, -3))`
- **THEN** it returns all 9 `ChunkCoord` values with `x` in `1..=3` and `y` in `-4..=-2`

### Requirement: Out-of-range enemies and debris are offloaded with distance hysteresis
While `SceneState` is `Game`, `stream_chunks` SHALL run in `FixedUpdate` in `SimulationSet::Cleanup`, ordered `.after(update_lifetime)`, on a periodic `StreamingTimer` using `STREAM_INTERVAL_SECS`, and SHALL evaluate distances from live player-team ships:
- Any live non-player `Ship` or `Debris` farther than `DESPAWN_RADIUS` from the nearest live player-team ship SHALL be captured as a `ShipSnapshot` or `DebrisSnapshot` via the shared `StreamSnapshot` helpers, saved into its `ChunkCoord` in `WorldStore`, and despawned from ECS only after the save succeeds, without emitting `ShipDestroyed` or spawning loot.
- Any stored `ShipSnapshot` or `DebrisSnapshot` in the 3×3 chunk neighborhood of any live player-team ship whose saved position is within `LOAD_RADIUS` of a live player-team ship SHALL be removed from `WorldStore` and spawned into live ECS with full snapshot fidelity.
- Player-team ships SHALL NEVER be despawned into chunks.

#### Scenario: Enemy beyond despawn_radius is saved and despawned without triggering quest kills
- **WHEN** an enemy ship is farther than `despawn_radius` from all player-team ships
- **THEN** its `ShipSnapshot` is saved into its `ChunkCoord` in `WorldStore`, the live entity and its blocks are despawned, and no `ShipDestroyed` message is emitted

#### Scenario: Stored enemy within load_radius is loaded back into ECS
- **WHEN** a player-team ship moves within `load_radius` of a stored enemy in a 3×3 neighbor chunk (including across a 50,000 chunk boundary)
- **THEN** the enemy is removed from `WorldStore` and spawned into live ECS with matching transform, velocity, block health, turret/battery state, doctrine, skill, and AI state

#### Scenario: Hysteresis prevents thrashing between load_radius and despawn_radius
- **WHEN** an enemy is at a distance strictly between `load_radius` and `despawn_radius`
- **THEN** a live enemy remains live and a stored enemy remains stored

#### Scenario: Detached debris offloads and reloads with remaining lifetime
- **WHEN** a detached `Debris` body exceeds `despawn_radius` and the player later returns within `load_radius`
- **THEN** the debris is saved as `DebrisSnapshot`, despawned, and later respawned with its blocks and preserved `remaining_secs`

#### Scenario: Player-team ships stay live regardless of distance
- **WHEN** two player-team ships are farther apart than `despawn_radius`
- **THEN** neither player-team ship is despawned into `WorldStore`

### Requirement: Chunk tuning is compile-time constants with test-only invariant checks
Chunk tuning SHALL be defined as compile-time constants in `core/src/constant.rs`: `CHUNK_SIZE = 50_000.0`, `LOAD_RADIUS = 8_000.0`, `DESPAWN_RADIUS = 10_000.0`, and `STREAM_INTERVAL_SECS = 0.25`. Streaming helpers SHALL read the constants directly. There SHALL be no `chunk.ron` RON config asset, no `ChunkConfig` asset or `ChunkConfigValue` resource, no hot-reload bridge, and no runtime `validate()`, startup assert, or other runtime validation. Invariants — `DESPAWN_RADIUS > LOAD_RADIUS`, `LOAD_RADIUS < CHUNK_SIZE * 0.5`, and all values positive — SHALL be guarded by unit tests only.

#### Scenario: Unit tests guard the chunk tuning invariants
- **WHEN** unit tests assert the invariants against the compile-time constants
- **THEN** any violation of `DESPAWN_RADIUS > LOAD_RADIUS`, `LOAD_RADIUS < CHUNK_SIZE * 0.5`, or positivity fails the test with an explicit message, with no startup or runtime validation path

#### Scenario: Periodic streaming timer fires at the compile-time interval
- **WHEN** a unit test advances `Time` by `STREAM_INTERVAL_SECS + 0.01`
- **THEN** `stream_chunks` fires once for that interval

### Requirement: Offload is save-first and chunk store failures are reported loudly
The streaming system SHALL persist a live entity's snapshot into `WorldStore` before despawning it, and SHALL despawn an entity only when its chunk data was successfully persisted. Any failed chunk load, save, or restore operation SHALL emit a `ChunkStreamError` message carrying the `ChunkCoord` and a detail string. A handler system SHALL log every `ChunkStreamError` loudly so data loss is never silent. Ship and debris snapshot capture/restore SHALL share a `StreamSnapshot` trait and generic helpers without changing the persisted `ChunkData` format.

#### Scenario: Failed save keeps the live entity and reports the error
- **WHEN** `WorldStore::save_chunk` fails while offloading an enemy ship or debris
- **THEN** the live entity is NOT despawned, and a `ChunkStreamError` carrying its `ChunkCoord` and the failure detail is emitted and logged

#### Scenario: Failed load is reported instead of silently skipped
- **WHEN** `WorldStore::load_chunk` fails for a chunk in a live player-team ship's 3×3 neighborhood
- **THEN** a `ChunkStreamError` carrying that `ChunkCoord` and the failure detail is emitted and logged, and no corrupt entity is spawned
