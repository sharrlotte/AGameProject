## MODIFIED Requirements

### Requirement: Load is a state transition
Loading a saved world SHALL set `PendingWorld::Restore { slot_id }`, enter `SceneState::Loading`, let exiting `Game` despawn old entities, then enter `Game`, spawn from the validated world snapshot, restore camera, `ControlledShip`, `CameraTarget`, `ControlMode`, AI runtime and targets, ship cargo, power and weapon charges, and item drops with sprites, and clear `PendingWorld`. No fresh skirmish SHALL spawn while a restore intent is active.

#### Scenario: Continue from Home
- **WHEN** the player clicks Play on Home and a valid last save resolves
- **THEN** `PendingWorld::Restore` is set and `SceneState::Game` restores the saved world instead of spawning a fresh skirmish, restoring pilot/camera control targets and AI runtime

#### Scenario: Failed validation does not transition
- **WHEN** the resolved slot fails validation before scene change
- **THEN** `PendingWorld` remains empty and `SceneState` does not change

### Requirement: No manual save/load slot UI in Pause
`Pause` SHALL NOT render `SlotPanel` or manual `Save`/`Load` slot buttons and SHALL expose only `Resume`, `Settings`, and `Ship Editor` / `Back to Home` actions. Home MAY expose a `Worlds` button that opens the slot picker; this button SHALL NOT exist in Pause.

#### Scenario: Pause menu has no Save, Load, or Exit buttons
- **WHEN** the player pauses the game
- **THEN** only `Resume`, `Settings`, and `Ship Editor` / `Back to Home` buttons are shown

#### Scenario: Worlds is available only on Home
- **WHEN** the player is in Pause during gameplay
- **THEN** no Worlds or slot-picker entry is rendered
