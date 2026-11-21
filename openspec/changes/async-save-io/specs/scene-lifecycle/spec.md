## MODIFIED Requirements

### Requirement: Application exit is available only from Home
Application exit SHALL be exposed only through `Home`'s `Exit` button. `Pause` SHALL NOT offer an application-exit action, and `PendingExitAction` SHALL NOT contain an exit variant. `PendingExitAction` SHALL support `None`, `Home`, and `ShipEditor` so both `Back to Home` and `Pause → Ship Editor` defer scene exit until the async save and flush complete, preventing `DespawnOnExit(SceneState::Game)` from despawning live ships before capture.

#### Scenario: Quitting during a battle
- **WHEN** the player wants to quit while in `SceneState::Game` or `SceneState::ShipEditor`
- **THEN** they use **Back to Home** first and then `Home`'s **Exit** button

#### Scenario: Deferred exit actions after an async save
- **WHEN** the player clicks **Back to Home** or **Ship Editor** from Pause in an active save slot
- **THEN** `PendingExitAction` records `Home` or `ShipEditor`, the live world is captured and saved asynchronously with WAL flush, and `SceneState` transitions only after `WorldSaved` is received
