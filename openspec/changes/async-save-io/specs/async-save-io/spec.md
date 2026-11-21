## ADDED Requirements

### Requirement: Disk persistence executes on a dedicated FIFO I/O worker thread
All `postcard` encoding, `lz4_flex` compression/decompression, SQLite transactions, slot file opens/creates, design persistence, and WAL checkpoints SHALL execute on a dedicated background `IoWorker` thread communicating through a bounded command channel and a result channel polled in `Update`. The main thread SHALL NOT perform blocking SQLite or filesystem I/O during gameplay frames.

#### Scenario: Autosave runs off the main thread
- **WHEN** `tick_auto_save` emits `SaveWorldEvent` during active gameplay
- **THEN** the main thread captures owned `CapturedWorld` data, submits `IoCommand::SaveWorld` to `IoWorker`, sets `SaveState::Saving`, and returns without performing encoding, compression, or SQLite writes on the main thread

#### Scenario: Worker executes queued commands in FIFO order
- **WHEN** a `SaveDesign` command is enqueued followed immediately by `SaveWorld` and `Flush`
- **THEN** `IoWorker` commits the design write before the world save and checkpoints the WAL last

#### Scenario: Worker shuts down cleanly on drop
- **WHEN** `IoWorker` is dropped on application exit
- **THEN** it drains pending commands, checkpoints the active SQLite store, and joins the worker thread without panicking

### Requirement: SaveState prevents overlapping saves and loads and signals completion
The save system SHALL maintain a `SaveState::{Idle, Saving { seq }, Loading { seq }}` resource and emit completion messages (`WorldSaved`, `WorldLoaded`, `SaveIoFailed`). Duplicate save or load requests arriving while busy SHALL be rejected or ignored for autosaves (never spawning concurrent writes), and stale results whose `seq` no longer matches the active operation after a scene cancellation SHALL be ignored cleanly.

#### Scenario: Duplicate autosave while saving is ignored
- **WHEN** `SaveWorldEvent` fires while `SaveState` is already `Saving`
- **THEN** no second save command is queued and the in-flight save completes uninterrupted

#### Scenario: Completed save updates SaveState and emits WorldSaved
- **WHEN** `IoWorker` finishes a world save and posts its result
- **THEN** `poll_io_results` transitions `SaveState` back to `Idle` and emits `WorldSaved`

#### Scenario: Failed save reports loudly without losing previous state
- **WHEN** `IoWorker` returns a `WorldStoreError` for a save or load
- **THEN** `SaveState` returns to `Idle`, `SaveIoFailed` is emitted, an `AppNotice` names the error, and the previously committed save file remains intact
