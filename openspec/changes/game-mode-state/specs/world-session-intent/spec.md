## MODIFIED Requirements

### Requirement: One world context owns identity, store, and intent
The system SHALL represent the active world as one `WorldContext` resource with variants `Skirmish { store }`, `Campaign { slot_id, path, store }`, and `Restoring { slot_id, path, store }`, synchronized with the `GameMode::{Skirmish, Campaign}` Bevy state in `WorldSavePlugin`. `PendingLoad`, `PendingWorld`, `WorldSession`, and `ActiveWorldStore` SHALL NOT exist. The store variant SHALL always match the context variant, and `Restoring` SHALL be converted to `Campaign` after a successful restore.

#### Scenario: Fresh skirmish uses the in-memory store and Skirmish mode
- **WHEN** QuickBattle / Skirmish starts
- **THEN** `WorldContext::Skirmish` holds the in-memory store, `GameMode` is `Skirmish`, no slot database is opened, and the sandbox stays in memory

#### Scenario: Restore names exactly one slot and sets Campaign mode
- **WHEN** the player continues a save
- **THEN** `WorldContext::Restoring` holds that slot's id, path, and store, `GameMode` transitions to `Campaign`, and no other slot is opened or written

### Requirement: Sandbox sessions are never persisted
`WorldContext::Skirmish` (`GameMode::Skirmish`) SHALL always hold `InMemoryWorldStore`. No autosave, world save, or design save from a `Skirmish` session SHALL write to `<data_dir>/saves/*.db`.

#### Scenario: Skirmish gameplay never triggers autosave
- **WHEN** gameplay runs in `GameMode::Skirmish`
- **THEN** `CampaignGame` is inactive, `tick_auto_save` does not run, and no file is created or modified on disk
