## ADDED Requirements

### Requirement: World space is partitioned into 50,000 x 50,000 storage chunks
The chunk streaming system SHALL map any 2D world position to a `ChunkCoord(IVec2)` using floor division by `ChunkConfig.chunk_size` (default `50_000.0`). Each `ChunkCoord` SHALL support computing its 3×3 neighborhood (itself and 8 adjacent chunks).

#### Scenario: Positive and negative coordinates map to consistent chunk coordinates
- **WHEN** `ChunkCoord::from_world` is called with `(-1.0, 0.0)` and `(0.0, 0.0)` at `chunk_size = 50_000.0`
- **THEN** `(-1.0, 0.0)` maps to `ChunkCoord(IVec2::new(-1, 0))` and `(0.0, 0.0)` maps to `ChunkCoord(IVec2::new(0, 0))`

#### Scenario: 3x3 neighborhood includes all 9 surrounding chunks
- **WHEN** `neighborhood_3x3` is called on `ChunkCoord(IVec2::new(2, -3))`
- **THEN** it returns all 9 `ChunkCoord` values with `x` in `1..=3` and `y` in `-4..=-2`

### Requirement: Out-of-range enemies and debris are offloaded with distance hysteresis
While `SceneState` is `Game`, the streaming system SHALL periodically evaluate distances from live player-team ships:
- Any live non-player `Ship` or `Debris` farther than `despawn_radius` from the nearest live player-team ship SHALL be captured as a `ShipSnapshot` or `DebrisSnapshot`, saved into its `ChunkCoord` in `WorldStore`, and despawned from ECS without emitting `ShipDestroyed` or spawning loot.
- Any stored `ShipSnapshot` or `DebrisSnapshot` in the 3×3 chunk neighborhood of any live player-team ship whose saved position is within `load_radius` of a live player-team ship SHALL be removed from `WorldStore` and spawned into live ECS with full snapshot fidelity.
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

### Requirement: Chunk streaming configuration is data-driven and validated
`ChunkConfig` SHALL be loaded from `core/assets/config/base/chunk.ron` with hot-reload support and validated so that `chunk_size > 0.0`, `load_radius > 0.0`, `despawn_radius > load_radius`, `load_radius < chunk_size * 0.5`, and `stream_interval_secs > 0.0`.

#### Scenario: Invalid hysteresis or oversized load_radius is rejected
- **WHEN** `ChunkConfig` has `despawn_radius <= load_radius` or `load_radius >= chunk_size * 0.5`
- **THEN** validation returns an explicit error
