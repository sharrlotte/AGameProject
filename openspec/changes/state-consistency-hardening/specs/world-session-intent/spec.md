## ADDED Requirements

### Requirement: Entering Game carries explicit world intent
The system SHALL represent pending world entry as a `PendingWorld` resource with at most one active intent: `None` for a fresh skirmish, or `Restore { slot_id }` for a saved world. `PendingLoad` SHALL NOT exist. The intent SHALL be set only after the target slot has been resolved and validated, and SHALL be cleared once the world has spawned.

#### Scenario: Fresh skirmish carries no restore intent
- **WHEN** QuickBattle starts
- **THEN** `PendingWorld` is `None`, no slot database is opened, and the sandbox stays in memory

#### Scenario: Restore intent names exactly one slot
- **WHEN** the player continues a save
- **THEN** `PendingWorld::Restore` holds that slot's id and no other slot is opened or written

### Requirement: Play continues the last save
The Play action on both Home and the Ship Editor SHALL resolve a target slot in this order: the active `WorldSession::SaveSlot` id; otherwise the slot with the greatest `updated_at_ms` returned by `WorldStore::list_slots()`. The resolved world SHALL be restored into `SceneState::Game`.

#### Scenario: Play inside an active slot resumes that slot
- **WHEN** the player opens the editor from Pause in Slot A and clicks Play
- **THEN** Slot A is restored, its session and store remain active, and no fresh skirmish is spawned

#### Scenario: Play from Home without an active slot uses the most recent slot
- **WHEN** several slots exist and none is active
- **THEN** the slot with the newest `updated_at_ms` is restored

### Requirement: Play creates a save when none exists
If no valid slot can be resolved, Play SHALL create a new save slot with a generated name, a random seed, and starter designs, then enter that world. Sandbox designs and QuickBattle state SHALL NOT be copied into the new slot.

#### Scenario: First launch with no save files
- **WHEN** the player clicks Play and the saves directory is empty
- **THEN** one new `.db` slot is created with a visible generated name and starter fleet, and the game enters that world

### Requirement: A failed continue never creates or replaces a save
If resolving or validating the target slot fails, the system SHALL show a notice naming the reason, SHALL keep the current scene unchanged, and SHALL NOT create a new slot or overwrite the failed one.

#### Scenario: Corrupt most-recent slot
- **WHEN** Play resolves to a corrupt slot and validation fails
- **THEN** the player stays on the current screen, the slot is unchanged, and the notice names the validation error

### Requirement: Home exposes a separate Worlds entry point
Home SHALL provide a Worlds button that opens the slot picker for creating named slots, loading a different slot, and deleting slots. Play SHALL NOT open the picker.

#### Scenario: Loading a different slot
- **WHEN** the player wants a slot other than the most recent one
- **THEN** Worlds opens the picker and the selected slot is loaded from there

### Requirement: Sandbox sessions are never persisted
`WorldSession::QuickBattle` SHALL always use `InMemoryWorldStore`. No autosave, world save, or design save from a sandbox session SHALL write to `<data_dir>/saves/*.db`.

#### Scenario: QuickBattle autosave tick
- **WHEN** the autosave timer fires during QuickBattle
- **THEN** no file is created or modified on disk
