## MODIFIED Requirements

### Requirement: Play continues the last save
The Play action on both Home and the Ship Editor SHALL resolve a target slot in this order: the active save slot id if one is open; otherwise the slot with the greatest `updated_at_ms` discovered by scanning `<data_dir>/saves/*.db` on the I/O worker (never by calling `InMemoryWorldStore::list_slots()`). The resolved slot SHALL be opened and read once asynchronously on `IoWorker`, validated on the main thread, and restored into `SceneState::Game` without reading the database twice.

#### Scenario: Play inside an active slot resumes that slot
- **WHEN** the player opens the editor from Pause in Slot A and clicks Play
- **THEN** Slot A is restored asynchronously, its session and store remain active, and no fresh skirmish is spawned

#### Scenario: Play from Home on a fresh launch finds the most recent slot on disk
- **WHEN** the application starts on Home (defaulting to an in-memory context), multiple valid `.db` slots exist under `<data_dir>/saves/`, and the player clicks Play
- **THEN** the I/O worker scans `<data_dir>/saves/*.db`, loads the slot with the newest `updated_at_ms` in a single read pass, validates it, and restores it into `SceneState::Game` without creating a new slot
