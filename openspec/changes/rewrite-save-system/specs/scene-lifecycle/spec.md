## MODIFIED Requirements

### Requirement: Load is a state transition

Loading the saved battle SHALL set `PendingLoad`, enter `Loading`, let exiting `Game` despawn old entities, enter `Game` and spawn from `battle.ron` instead of the mission spawn path, restore battle state (including `BattlePhase::Ended` when an outcome is saved), camera, `ControlledShip`, `CameraTarget`, `ControlMode`, AI runtime and targets, ship cargo, power/weapon charges, and item drops with sprites, and clear `PendingLoad`.

#### Scenario: Continue from Home
- **WHEN** the player clicks `Continue` on `Home`
- **THEN** `PendingLoad` is set and `SceneState::Game` spawns from `battle.ron` instead of a fresh mission, restoring pilot/camera control targets and battle phase
