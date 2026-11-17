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

Outcome detection SHALL run in `FixedPostUpdate` in `SimulationSet::Death`, after
`apply_ship_mutations`. It SHALL consume `ShipDestroyed` as its trigger and SHALL determine
remaining ships by querying `Query<&Team, With<Ship>>` at that moment, rather than by maintained
counters. The first side with zero ships SHALL set `BattleResult.outcome` and transition
`BattlePhase` to `Ended`.

#### Scenario: Player core destroyed
- **WHEN** `ShipDestroyed` is received for a player ship and no player ships remain
- **THEN** `BattleResult.outcome` is `Defeat` and `BattlePhase` becomes `Ended`

#### Scenario: Enemy core destroyed
- **WHEN** `ShipDestroyed` is received for an enemy ship and no enemy ships remain
- **THEN** `BattleResult.outcome` is `Victory` and `BattlePhase` becomes `Ended`

#### Scenario: Other ships remain
- **WHEN** a `ShipDestroyed` message is received but both teams still have at least one ship
- **THEN** `BattleResult.outcome` remains unchanged and `BattlePhase` stays `Active`

### Requirement: Mission logic is scoped to the Game scene

Outcome detection and mission loading SHALL only run while `SceneState` is `Game`. Leaving Game
SHALL set `BattlePhase` to `Idle` (the default state) and reset `BattleResult`; `BattleRestart`
SHALL be preserved so a Retry request is still honoured on entering Home.

#### Scenario: Back to Home from the pause menu
- **WHEN** the player clicks "Back to Home" while `BattlePhase` is `Active`
- **THEN** scene teardown despawns the ships without emitting `ShipDestroyed`, no outcome is
  recorded, and no DEFEAT/VICTORY overlay is spawned over the Home menu

#### Scenario: Mission asset finishes loading after scene exit
- **WHEN** `poll_mission_loaded` would complete after the player has left Game
- **THEN** the scene guard prevents any ships from spawning into the Home scene

#### Scenario: Retry after a completed battle
- **WHEN** the overlay's Retry button was pressed before leaving Game
- **THEN** `BattleRestart` survives the Game-exit reset and `resume_battle_on_restart` re-enters
  Game on `OnEnter(Home)`

### Requirement: The end overlay remains state-driven

The DEFEAT/VICTORY overlay SHALL spawn only on `OnEnter(BattlePhase::Ended)` and SHALL read its
title and duration from `BattleResult`. It SHALL NOT subscribe directly to `ShipDestroyed`.

#### Scenario: Overlay content
- **WHEN** `BattlePhase` enters `Ended` with outcome `Defeat`
- **THEN** an overlay showing "DEFEAT" and the recorded duration is spawned
