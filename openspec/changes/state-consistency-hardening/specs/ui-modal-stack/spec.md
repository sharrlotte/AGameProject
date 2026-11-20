## ADDED Requirements

### Requirement: Modal UI has a single owner
The system SHALL represent Pause, Settings, and SlotPicker overlays as variants of one `ModalStack` resource. `PauseState`, `SettingState`, and `SlotPickerState` SHALL NOT exist, and no Bevy `State` SHALL independently control a modal overlay.

#### Scenario: Settings opened over pause
- **WHEN** the player pauses gameplay and clicks Settings
- **THEN** the stack holds both Pause and Settings, with Settings on top, and exactly one settings overlay exists

#### Scenario: No orphaned state after reload
- **WHEN** a saved world is restored
- **THEN** modal visibility is derived only from `ModalStack`, not from any state left over from the previous scene

### Requirement: Escape pops only the topmost modal
Escape SHALL remove the top modal from `ModalStack`, or toggle Pause when the stack is empty and the scene is Game or ShipEditor. Escape SHALL NOT unpause gameplay while Settings or SlotPicker is on top.

#### Scenario: Escape closes Settings first
- **WHEN** Settings is open over Pause and the player presses Escape
- **THEN** Settings is popped, Pause remains on top, and virtual time stays paused

#### Scenario: Escape with no modal toggles pause
- **WHEN** the stack is empty during gameplay and the player presses Escape
- **THEN** Pause is pushed and virtual time is paused

### Requirement: Leaving Game clears all modals and restores time
On exiting `SceneState::Game` the system SHALL clear `ModalStack`, unpause `Time<Virtual>`, and despawn every modal-owned UI entity. No modal overlay SHALL survive into Home or ShipEditor.

#### Scenario: Mission failure while paused returns home cleanly
- **WHEN** the player is paused and the game transitions to Home
- **THEN** the pause overlay is despawned, `ModalStack` is empty, and virtual time is running

#### Scenario: Re-entering Game starts with no modal
- **WHEN** the player continues a save from Home
- **THEN** gameplay starts with an empty `ModalStack` and running virtual time

### Requirement: Virtual time pause follows the active gameplay modal
`Time<Virtual>` SHALL be paused only when the top modal is Pause and the scene is Game or ShipEditor. Opening SlotPicker from Home SHALL NOT pause virtual time.

#### Scenario: Slot picker on Home does not freeze time
- **WHEN** the player opens Worlds on Home
- **THEN** no virtual-time pause is applied and closing the picker requires no unpause step
