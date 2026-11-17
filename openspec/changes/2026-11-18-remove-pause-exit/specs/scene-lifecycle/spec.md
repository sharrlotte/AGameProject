## MODIFIED Requirements

### Requirement: No manual save/load slot UI in Home or Pause
`Home` and `Pause` SHALL NOT render `SlotPanel` or manual `Save`/`Load` slot buttons. `Pause` SHALL expose only `Resume`, `Settings`, and `Back to Home`.

#### Scenario: Pause menu has no Save, Load, or Exit buttons
- **WHEN** the player pauses the game
- **THEN** only `Resume`, `Settings`, and `Back to Home` buttons are shown

## ADDED Requirements

### Requirement: Application exit is available only from Home
Application exit SHALL be exposed only through `Home`'s `Exit` button. `Pause` SHALL NOT offer an application-exit action, and `PendingExitAction` SHALL NOT contain an exit variant.

#### Scenario: Quitting during a battle
- **WHEN** the player wants to quit while in `SceneState::Game` or `SceneState::ShipEditor`
- **THEN** they use **Back to Home** first and then `Home`'s **Exit** button

#### Scenario: Deferred exit actions after an auto-save
- **WHEN** a campaign auto-save completes with a pending exit action
- **THEN** the only supported action is returning to `SceneState::Home`
