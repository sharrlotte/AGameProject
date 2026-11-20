# Scene Lifecycle

## Purpose

State-scoped despawning, an explicit schedule contract, and state-driven loading keep scene transitions leak-free.
## Requirements
### Requirement: All scene entities are state-scoped

Gameplay, home, editor, pause, and settings entities SHALL use `DespawnOnExit` with their owning state; children SHALL despawn with parents. `SceneEntity`, `clean_state_item`, the `Save` marker, and manual scene cleanup loops SHALL NOT exist.

#### Scenario: Leaving the game cleans everything
- **WHEN** the game leaves `SceneState::Game`
- **THEN** no gameplay entity remains, including drops, damage text, bullets, lasers, beams, and UI

### Requirement: The schedule contract is explicit and shared

Systems SHALL be registered into the documented sets: `Input` in `PreUpdate`; `Stats` in `FixedPreUpdate`; `Ai`, `Control`, and `Weapons` in `FixedUpdate`; `Damage` and `Death` in `FixedPostUpdate`; `Present` in `PostUpdate`; and `Cleanup` in `Last`. Ad-hoc ordering SHALL NOT be used where set ordering is required.

#### Scenario: New plugin declares ordering
- **WHEN** a new plugin depends on another plugin's output
- **THEN** it declares the dependency through the shared sets

### Requirement: Damage, death, and ship mutation are chained

The pipeline SHALL order collision and damage events, health and death handling, and `BodyMutation` application explicitly within the same fixed tick, without relying on message persistence across frames.

#### Scenario: Lethal damage reaches mutation
- **WHEN** a block takes lethal damage
- **THEN** its block entity is removed and the body mutation removes its cells before the next frame

### Requirement: Load is a state transition
Loading a saved world SHALL set `WorldContext::Restoring { slot_id, path, store }`, enter `SceneState::Loading`, let exiting `Game` despawn old entities, then enter `Game`, spawn from the validated world snapshot, restore camera, `ControlledShip`, `CameraTarget`, `ControlMode`, AI runtime and targets, ship cargo, power and weapon charges, and item drops with sprites, and convert the context to `SaveSlot`. No fresh skirmish SHALL spawn while `Restoring` is active.

#### Scenario: Continue from Home
- **WHEN** the player clicks Play on Home and a valid last save resolves
- **THEN** `WorldContext::Restoring` is set and `SceneState::Game` restores the saved world instead of spawning a fresh skirmish, restoring pilot/camera control targets and AI runtime

#### Scenario: Failed validation does not transition
- **WHEN** the resolved slot fails validation before scene change
- **THEN** `WorldContext` remains unchanged and `SceneState` does not change

### Requirement: No manual save/load slot UI in Home or Pause
`Pause` SHALL NOT render `SlotPanel` or manual `Save`/`Load` slot buttons and SHALL expose only `Resume`, `Settings`, and `Ship Editor` / `Back to Home` actions. Home MAY expose a `Worlds` button that opens the slot picker; this button SHALL NOT exist in Pause.

#### Scenario: Pause menu has no Save, Load, or Exit buttons
- **WHEN** the player pauses the game
- **THEN** only `Resume`, `Settings`, and `Ship Editor` / `Back to Home` buttons are shown

#### Scenario: Worlds is available only on Home
- **WHEN** the player is in Pause during gameplay
- **THEN** no Worlds or slot-picker entry is rendered

### Requirement: Application exit is available only from Home
Application exit SHALL be exposed only through `Home`'s `Exit` button. `Pause` SHALL NOT offer an application-exit action, and `PendingExitAction` SHALL NOT contain an exit variant.

#### Scenario: Quitting during a battle
- **WHEN** the player wants to quit while in `SceneState::Game` or `SceneState::ShipEditor`
- **THEN** they use **Back to Home** first and then `Home`'s **Exit** button

#### Scenario: Deferred exit actions after an auto-save
- **WHEN** a campaign auto-save completes with a pending exit action
- **THEN** the only supported action is returning to `SceneState::Home`

### Requirement: Pilot and camera controls are separate

The game SHALL expose `Pilot` and `FreeLook` modes: WASD flies the controlled ship in `Pilot` and pans the camera in `FreeLook`; `F` toggles follow; middle-mouse drag pans in either mode; the default is `Pilot` when a ship is controlled and `FreeLook` otherwise.

#### Scenario: W in pilot mode
- **WHEN** the player presses W in `Pilot`
- **THEN** the ship thrusts and the camera does not pan independently

### Requirement: Untagged entities are audited in debug builds

A debug-only system SHALL compare `DespawnOnExit`-tagged roots with spawned gameplay roots and warn about untagged entities.

#### Scenario: Missing state scope
- **WHEN** a gameplay entity spawns without a state scope
- **THEN** the debug audit reports it

