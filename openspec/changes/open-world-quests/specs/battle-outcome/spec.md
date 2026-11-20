## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Mission logic is scoped to the Game scene
**Reason**: Arena missions (`core/src/mission/`, `MissionDef`, `BattlePhase`, `BattleRestart`) are replaced by open-world quests and save slots.
**Migration**: Quest systems, generation, and respawn are scoped to `SceneState::Game` in `core/src/quest/`; quest progression is additionally gated on `WorldSession::SaveSlot`.

### Requirement: The end overlay remains state-driven
**Reason**: Arena Victory/Defeat end overlays are removed in favor of continuous open-world play with respawn on player defeat.
**Migration**: Player defeat triggers respawn in `core/src/quest/system.rs`; no end overlay is spawned.
