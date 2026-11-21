## ADDED Requirements

### Requirement: GameMode state and CampaignGame computed state replace is_save_slot gating
The engine SHALL register a top-level Bevy 0.20 `States` enum `GameMode::{Skirmish, Campaign}` (default `Skirmish`) and a `ComputedStates` `CampaignGame` derived from `(SceneState, GameMode)` that exists if and only if `(SceneState::Game, GameMode::Campaign)`. Campaign-only gameplay systems (quest progression, quest generation, quest HUD, auto-save) SHALL be gated via `in_state(CampaignGame)`, `OnEnter(CampaignGame)`, `OnExit(CampaignGame)`, and `DespawnOnExit(CampaignGame)`. Pause and Ship Editor mode checks SHALL read `State<GameMode>`. `save_slot_only` and inline `context.is_save_slot()` guards SHALL NOT exist.

#### Scenario: Entering Campaign gameplay activates CampaignGame and spawns Quest HUD
- **WHEN** `GameMode` transitions to `Campaign` and `SceneState` transitions to `Game`
- **THEN** `CampaignGame` becomes active, Quest HUD entities spawn tagged with `DespawnOnExit(CampaignGame)`, and quest + auto-save systems run

#### Scenario: Entering Skirmish gameplay never activates CampaignGame or spawns Quest HUD
- **WHEN** `GameMode` is `Skirmish` and `SceneState` transitions to `Game`
- **THEN** `CampaignGame` is inactive, zero Quest HUD entities spawn, and no quest progression or autosave systems execute

#### Scenario: GameMode stays Campaign across Game to ShipEditor transition
- **WHEN** the player opens Ship Editor from Pause during a `Campaign` session
- **THEN** `CampaignGame` deactivates (despawning Quest HUD via `DespawnOnExit(CampaignGame)`), while `State<GameMode>` remains `GameMode::Campaign` until returning to `SceneState::Home` or starting `Skirmish`
