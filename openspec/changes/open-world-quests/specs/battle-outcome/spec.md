## MODIFIED Requirements

### Requirement: Battle outcome is decided from the destruction message
In open-world gameplay (`SceneState::Game`), `ShipDestroyed` SHALL NOT trigger an arena `Victory` screen when enemy ships reach zero. When `ShipDestroyed` is received for a player-team ship and zero live player-team ships remain, the game SHALL respawn a fresh starter player ship at the configured respawn point (default `(0, 0)`), re-attach player control and camera target, and preserve all world state and quest progress.

#### Scenario: Destroying all nearby enemies does not end the game
- **WHEN** `ShipDestroyed` is received for the last live enemy ship
- **THEN** gameplay continues normally with no Victory overlay

#### Scenario: Last player ship destroyed triggers origin respawn
- **WHEN** `ShipDestroyed` is received for the last live player-team ship
- **THEN** a fresh starter player ship spawns at `(0, 0)` with control and camera attached, and existing world entities and `QuestTracker` progress remain intact

## REMOVED Requirements

### Requirement: Mission logic is scoped to the Game scene
**Reason**: Arena missions (`core/src/mission/`, `MissionDef`, `BattlePhase`, `BattleRestart`) are replaced by open-world quests (`open-world-quests`) and save slots (`world-save-slots`).
**Migration**: Quest systems and open-world respawn are scoped to `SceneState::Game` in `core/src/quest/`.

### Requirement: The end overlay remains state-driven
**Reason**: Arena Victory/Defeat end overlays are removed in favor of continuous open-world play with origin respawn on player defeat.
**Migration**: Player defeat triggers origin respawn in `core/src/quest/system.rs`.
