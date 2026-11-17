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

The pipeline SHALL order collision and damage events, health and death handling, and `ShipMutation` application explicitly within the same fixed tick, without relying on message persistence across frames.

#### Scenario: Lethal damage reaches mutation
- **WHEN** a block takes lethal damage
- **THEN** its block entity is marked `Death` and the ship mutation removes its cells before the next frame

### Requirement: Load is a state transition

Loading SHALL set `PendingLoad`, enter `Loading`, let exiting `Game` despawn old entities, enter `Game` and spawn from the snapshot instead of the mission spawn path, restore battle, camera, AI, cargo, and power, and clear `PendingLoad`.

#### Scenario: Pause-menu load
- **WHEN** a slot is loaded from the pause menu
- **THEN** no old game entity survives and the snapshot is the only source of ships

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
