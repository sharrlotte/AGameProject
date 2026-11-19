## MODIFIED Requirements

### Requirement: Battle outcome is decided from the destruction message

Outcome detection SHALL run in `FixedPostUpdate` in `SimulationSet::Death`, after
`apply_body_mutations`. It SHALL consume `ShipDestroyed` as its trigger and SHALL determine
remaining live ships by querying `Query<(Entity, &Team), With<Ship>>` at that moment, combined
with unloaded enemy ships persisted in `ChunkStore::total_enemy_count()`. The first side with
zero remaining ships (live plus unloaded for enemies) SHALL set `BattleResult.outcome` and
transition `BattlePhase` to `Ended`.

#### Scenario: Player core destroyed
- **WHEN** `ShipDestroyed` is received for a player ship and no player ships remain
- **THEN** `BattleResult.outcome` is `Defeat` and `BattlePhase` becomes `Ended`

#### Scenario: Enemy core destroyed with no unloaded enemies remaining
- **WHEN** `ShipDestroyed` is received for an enemy ship, no live enemy ships remain, and `ChunkStore::total_enemy_count()` is `0`
- **THEN** `BattleResult.outcome` is `Victory` and `BattlePhase` becomes `Ended`

#### Scenario: Enemy core destroyed while unloaded enemies remain in chunks
- **WHEN** `ShipDestroyed` is received for the last live enemy ship, but `ChunkStore::total_enemy_count()` is greater than `0`
- **THEN** `BattleResult.outcome` remains `None` and `BattlePhase` stays `Active`

#### Scenario: Other ships remain
- **WHEN** a `ShipDestroyed` message is received but both teams still have at least one live ship
- **THEN** `BattleResult.outcome` remains unchanged and `BattlePhase` stays `Active`
