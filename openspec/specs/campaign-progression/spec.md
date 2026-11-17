# Campaign Progression

## Purpose

Single-save campaign progression with automatic battle snapshots, stateless Quick Battle isolation, and Main-Menu-only progress reset.

## Requirements

### Requirement: Campaign progression is persisted separately from active battle state
The system SHALL persist campaign progression in `<data_dir>/saves/campaign.ron` as a versioned RON document (`CampaignSave`), distinct from the active mid-battle snapshot `<data_dir>/saves/battle.ron`.

#### Scenario: Victory records mission completion
- **WHEN** a `Campaign` battle reaches `BattlePhase::Ended` with `BattleOutcome::Victory`
- **THEN** the mission is recorded in `campaign.ron` and `battle.ron` is removed

#### Scenario: Defeat clears active battle snapshot
- **WHEN** a `Campaign` battle reaches `BattlePhase::Ended` with `BattleOutcome::Defeat`
- **THEN** `battle.ron` is removed so the player starts the mission fresh next time

### Requirement: Campaign battles auto-save automatically
When `BattleMode::Campaign` is active in `SceneState::Game`, the system SHALL automatically write `battle.ron` without manual save UI.

#### Scenario: Periodic auto-save during active battle
- **WHEN** `BattlePhase::Active` and `PauseState::Playing` are active in `BattleMode::Campaign` and the configured `.ron` auto-save interval elapses
- **THEN** the current battle snapshot is written to `battle.ron`

#### Scenario: Auto-save on Pause Back to Home
- **WHEN** the player clicks `Back to Home` in the Pause menu during a `Campaign` battle
- **THEN** the current battle snapshot is written to `battle.ron` before `SceneState::Game` is exited

### Requirement: Quick Battle is stateless
`BattleMode::QuickBattle` SHALL NOT read, overwrite, or delete `battle.ron` or `campaign.ron`.

#### Scenario: Quick Battle leaves campaign save untouched
- **WHEN** the player launches `Quick Battle`, fights, pauses to Home, or finishes the battle
- **THEN** neither `battle.ron` nor `campaign.ron` is created, modified, or deleted

### Requirement: Home menu adapts Play vs Continue to active battle save
The `Home` menu SHALL display **Continue** when `battle.ron` exists and **Play** when no active battle save exists, while keeping **Quick Battle** as a separate button.

#### Scenario: Active battle save exists on Home entry
- **WHEN** `SceneState::Home` is entered and `battle.ron` exists on disk
- **THEN** the primary launch button displays `Continue` and clicking it loads `battle.ron` into `SceneState::Game` in `BattleMode::Campaign`

#### Scenario: No active battle save exists on Home entry
- **WHEN** `SceneState::Home` is entered and `battle.ron` does not exist
- **THEN** the primary launch button displays `Play` and clicking it starts a fresh `Campaign` mission in `SceneState::Game`

### Requirement: Main Menu Settings includes a Data reset section
When `SettingState::Open` is entered from `SceneState::Home`, the Settings UI SHALL display a **Data** section with a **Reset Mission / Campaign** action protected by a 2-step confirmation. This section SHALL NOT be shown when Settings is opened from `Pause` during `SceneState::Game` or `SceneState::ShipEditor`.

#### Scenario: Reset requires 2-step confirmation
- **WHEN** the player clicks `Reset Mission / Campaign` in Main Menu Settings
- **THEN** the UI prompts for confirmation (`Confirm` / `Cancel`) without deleting any files yet

#### Scenario: Confirming reset clears progress but preserves ship designs
- **WHEN** the player confirms the reset in Main Menu Settings
- **THEN** `battle.ron` and `campaign.ron` are deleted, in-memory campaign state is reset, `Home` updates `Continue` back to `Play`, and player ship designs in `designs/*.ron` remain untouched

#### Scenario: Data section hidden outside Home
- **WHEN** the player opens Settings from the Pause menu in `SceneState::Game`
- **THEN** the Data reset section is not displayed
