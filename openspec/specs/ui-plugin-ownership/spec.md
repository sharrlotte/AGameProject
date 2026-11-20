# ui-plugin-ownership Specification

## Purpose
TBD - created by archiving change runtime-state-cleanup. Update Purpose after archive.
## Requirements
### Requirement: UI systems are owned by a UI plugin
`UiPlugin` in `ui/mod.rs` SHALL initialize `ModalStack` and register all modal lifecycle systems. `main.rs` SHALL add the plugin but SHALL NOT register modal systems directly.

#### Scenario: Modal systems resolve from one plugin
- **WHEN** the app is built
- **THEN** modal initialization, clearing, and time syncing all come from `UiPlugin`

### Requirement: Scene transitions clear modals without polling
`UiPlugin` SHALL clear `ModalStack` and unpause `Time<Virtual>` on `OnExit(SceneState::Game)`. No system SHALL detect scene changes by storing the previous scene in a `Local`.

#### Scenario: Leaving Game while paused
- **WHEN** the game transitions out of `SceneState::Game` with a modal open
- **THEN** the stack is cleared and virtual time is running in the destination scene

### Requirement: Modal time sync is scheduled and change-driven
Virtual-time synchronization SHALL run in `SimulationSet::Present` and SHALL be gated by `resource_changed::<ModalStack>`.

#### Scenario: No modal change
- **WHEN** the modal stack is unchanged for several frames
- **THEN** the time-sync system does not run

