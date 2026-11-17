## MODIFIED Requirements

### Requirement: Campaign battles auto-save automatically
When `BattleMode::Campaign` is active in `SceneState::Game`, the system SHALL automatically write `battle.ron` without manual save UI.

#### Scenario: Periodic auto-save during active battle
- **WHEN** `BattlePhase::Active` and `PauseState::Playing` are active in `BattleMode::Campaign` and the configured `.ron` auto-save interval elapses
- **THEN** the current battle snapshot is written to `battle.ron`

#### Scenario: Auto-save on Pause Back to Home
- **WHEN** the player clicks `Back to Home` in the Pause menu during a `Campaign` battle
- **THEN** the current battle snapshot is written to `battle.ron` before `SceneState::Game` is exited
