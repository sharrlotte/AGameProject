# Battle Outcome

## Purpose

Battle outcomes are derived from an explicit core-destruction message rather than from ship
counts that cannot distinguish a real kill from scene teardown. Mission logic is scoped to the
Game scene and has an explicit resting state.
## Requirements
### Requirement: Core destruction emits a ship-level message

The mutation pipeline SHALL emit exactly one `ShipDestroyed` message when a ship's core is
removed and the grid can no longer produce a valid ship. The message SHALL carry the ship
entity and its team. Ships already carrying the `Death` marker SHALL be excluded from mutation
processing and SHALL NOT emit.

#### Scenario: Last core block is destroyed
- **WHEN** a `DestroyBlock` mutation removes a ship's only core and
  `build_ship_result` returns no ship
- **THEN** the ship receives `Death` and one `ShipDestroyed { ship, team }` message is written

#### Scenario: Core-less ship is mutated again
- **WHEN** a second mutation targets a ship that already carries `Death`
- **THEN** no further `ShipDestroyed` message is emitted for that ship

### Requirement: Battle outcome is decided from the destruction message

Outcome detection SHALL run in `FixedPostUpdate` in `SimulationSet::Death`, after `apply_body_mutations`. It SHALL consume `ShipDestroyed` as its trigger and SHALL determine remaining ships by querying `Query<&Team, With<Ship>>` at that moment, rather than by maintained counters. Enemy destruction SHALL NOT end gameplay or produce a Victory screen. When `ShipDestroyed` is received for a player-team ship and zero live player-team ships remain, the system SHALL respawn a fresh starter player ship at the configured respawn point (default `(0, 0)`), re-attach `ControlledShip` and `CameraTarget`, and preserve all world entities, chunks, and quest progress. This behavior SHALL apply in both save slots and `QuickBattle`.

#### Scenario: Destroying all nearby enemies does not end the game
- **WHEN** `ShipDestroyed` is received for the last live enemy ship
- **THEN** gameplay continues with no Victory overlay and no end transition

#### Scenario: Last player ship destroyed triggers origin respawn
- **WHEN** `ShipDestroyed` is received for the last live player-team ship
- **THEN** a starter player ship spawns at the respawn point with control and camera attached, and existing world entities and quest progress remain intact

#### Scenario: Other ships remain
- **WHEN** a `ShipDestroyed` message is received but at least one player-team ship remains
- **THEN** no respawn occurs and gameplay continues

#### Scenario: Respawn in QuickBattle
- **WHEN** the last player ship is destroyed during `QuickBattle`
- **THEN** a starter player ship is respawned and the skirmish continues

