# world-session-intent Specification

## Purpose
TBD - created by archiving change state-consistency-hardening. Update Purpose after archive.
## Requirements
### Requirement: One world context owns identity, store, and intent
The system SHALL represent the active world as one `WorldContext` resource with variants `QuickBattle { store }`, `SaveSlot { slot_id, path, store }`, and `Restoring { slot_id, path, store }`. `PendingLoad`, `PendingWorld`, `WorldSession`, and `ActiveWorldStore` SHALL NOT exist. The store variant SHALL always match the session variant, and `Restoring` SHALL be converted to `SaveSlot` after a successful restore.

#### Scenario: Fresh skirmish uses the in-memory store
- **WHEN** QuickBattle starts
- **THEN** `WorldContext::QuickBattle` holds the in-memory store, no slot database is opened, and the sandbox stays in memory

#### Scenario: Restore names exactly one slot and its store
- **WHEN** the player continues a save
- **THEN** `WorldContext::Restoring` holds that slot's id, path, and store, and no other slot is opened or written

### Requirement: Play continues the last save
The Play action on both Home and the Ship Editor SHALL resolve a target slot in this order: the active `WorldContext::SaveSlot` or `Restoring` id; otherwise the slot with the greatest `updated_at_ms` returned by `WorldStore::list_slots()`. The resolved world SHALL be restored into `SceneState::Game`.

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
`WorldContext::QuickBattle` SHALL always hold `InMemoryWorldStore`. No autosave, world save, or design save from a sandbox session SHALL write to `<data_dir>/saves/*.db`.

#### Scenario: QuickBattle autosave tick
- **WHEN** the autosave timer fires during QuickBattle
- **THEN** no file is created or modified on disk

