## MODIFIED Requirements

### Requirement: Compact HUD Quest Tracker displays active objectives

While `CampaignGame` (`(SceneState::Game, GameMode::Campaign)`) is active, a compact HUD panel tagged with `DespawnOnExit(CampaignGame)` SHALL display active quests and their objective progress counters, updating reactively when quest progress changes. The manual-offer notification queue SHALL be a separate right-center panel also scoped to `DespawnOnExit(CampaignGame)`. Neither panel SHALL spawn in `GameMode::Skirmish`.

#### Scenario: HUD updates when objective progress changes
- **WHEN** in `CampaignGame` an active quest's `DestroyEnemies` progress advances from `0/3` to `1/3`
- **THEN** the HUD Quest Tracker displays `1/3`

#### Scenario: Quest HUD is absent in Skirmish
- **WHEN** the player starts a `GameMode::Skirmish` session
- **THEN** neither the Quest Tracker nor the offer notification queue is spawned

### Requirement: QuickBattle remains a functional unsaved skirmish

`GameMode::Skirmish` (Quick Battle) SHALL spawn a player fleet and test enemies in memory without campaign persistence. Quest activation, objective progression, quest HUD, and autosave SHALL NOT run while `CampaignGame` is inactive. Player respawn SHALL still function in `GameMode::Skirmish`.

#### Scenario: Skirmish kills do not alter saved quest progress
- **WHEN** an enemy is destroyed during `GameMode::Skirmish`
- **THEN** `CampaignGame` is inactive and no quest objective advances

#### Scenario: Skirmish defeat respawns the player
- **WHEN** the last player ship is destroyed during `GameMode::Skirmish`
- **THEN** a starter player ship is respawned and play continues
