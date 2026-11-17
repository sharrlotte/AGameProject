## MODIFIED Requirements

### Requirement: Campaign progression is persisted separately from active battle state
The system SHALL persist campaign progression in `<data_dir>/saves/campaign.ron` as a versioned RON document (`CampaignSave`) backed by an in-memory `CampaignSave` Bevy `Resource`, distinct from the active mid-battle snapshot `<data_dir>/saves/battle.ron` tracked by the `BattleSaveState` Bevy `Resource`.

#### Scenario: Victory records mission completion
- **WHEN** a `Campaign` battle reaches `BattlePhase::Ended` with `BattleOutcome::Victory`
- **THEN** the mission is recorded in the `CampaignSave` resource and persisted to `campaign.ron`, `battle.ron` is removed, and `BattleSaveState` marks no active battle save

#### Scenario: Defeat clears active battle snapshot
- **WHEN** a `Campaign` battle reaches `BattlePhase::Ended` with `BattleOutcome::Defeat`
- **THEN** `battle.ron` is removed and `BattleSaveState` marks no active battle save so the player starts the mission fresh next time

### Requirement: Home menu adapts Play vs Continue to active battle save
The `Home` menu SHALL display **Continue** when `BattleSaveState` indicates an active `battle.ron` exists and **Play** when no active battle save exists, while keeping **Quick Battle** as a separate button.

#### Scenario: Active battle save exists on Home entry
- **WHEN** `SceneState::Home` is entered and `BattleSaveState` indicates `battle.ron` exists
- **THEN** the primary launch button displays `Continue` and clicking it loads `battle.ron` into `SceneState::Game` in `BattleMode::Campaign`

#### Scenario: No active battle save exists on Home entry
- **WHEN** `SceneState::Home` is entered and `BattleSaveState` indicates `battle.ron` does not exist
- **THEN** the primary launch button displays `Play` and clicking it starts a fresh `Campaign` mission in `SceneState::Game`

### Requirement: Main Menu Settings includes a Data reset section
When `SettingState::Open` is entered from `SceneState::Home`, the Settings UI SHALL display a **Data** section with a **Reset Mission / Campaign** action protected by a 2-step confirmation. This section SHALL NOT be shown when Settings is opened from `Pause` during `SceneState::Game` or `SceneState::ShipEditor`.

#### Scenario: Reset requires 2-step confirmation
- **WHEN** the player clicks `Reset Mission / Campaign` in Main Menu Settings
- **THEN** the UI prompts for confirmation (`Confirm` / `Cancel`) without deleting any files yet

#### Scenario: Confirming reset clears progress but preserves ship designs
- **WHEN** the player confirms the reset in Main Menu Settings
- **THEN** `battle.ron` and `campaign.ron` are deleted, the `CampaignSave` and `BattleSaveState` resources are reset in memory, `Home` updates `Continue` back to `Play`, and player ship designs in `designs/*.ron` and `ShipCatalog` remain untouched

#### Scenario: Data section hidden outside Home
- **WHEN** the player opens Settings from the Pause menu in `SceneState::Game`
- **THEN** the Data reset section is not displayed
