## 1. Implement `IoWorker` Thread and Channels

- [ ] 1.1 Create `core/src/save/world/worker.rs` defining `IoCommand`, `IoResult`, `CapturedWorld`, and `IoWorker` resource owning the bounded `sync_channel` sender, result receiver, and `JoinHandle`.
- [ ] 1.2 Move `postcard` + `lz4_flex` encoding (`WorldMeta`, `ActiveStateSnapshot`, `QuestStoreRow`) out of main-thread capture into the worker thread's `SaveWorld` handler.
- [ ] 1.3 Implement worker handlers for `AttachStore`, `DetachToMemory`, `SaveWorld` (with conditional `flush`), `OpenAndLoad`, `ResolveAndContinue`, `CreateSlot`, `SaveDesign`, `ListSlots`, and graceful `Shutdown` on `Drop`.

## 2. Add `SaveState`, Completion Messages, and Polling Systems

- [ ] 2.1 Add `SaveState::{Idle, Saving { seq }, Loading { seq }}` resource and `WorldSaved`, `WorldLoaded`, `SaveIoFailed` messages in `core/src/save/world/component.rs`.
- [ ] 2.2 Refactor `handle_save_world_event` (`core/src/save/world/system.rs`) to capture owned `CapturedWorld` in a single entity pass, skip when busy, and submit `IoCommand::SaveWorld` (`flush: false` for autosaves, `flush: true` for exit saves).
- [ ] 2.3 Refactor `handle_load_slot`, `handle_create_slot`, and `handle_continue_world` to dispatch asynchronously to `IoWorker`, scanning `saves_dir()` directly for `ContinueWorldEvent` and reading slot data once (removing double `load_world_data`).
- [ ] 2.4 Add `poll_io_results` in `Update` (`SimulationSet::Present`) to validate loaded snapshots against `BlockRegistry`, emit completion messages, update `SaveState`, show `AppNotice` on errors, and discard cancelled/stale sequence IDs.

## 3. Fix Scene Exit Race and Slot Picker Async Refresh

- [ ] 3.1 Extend `PendingExitAction` with `ShipEditor` and update `on_ship_editor` (`core/src/scene/pause/mod.rs`) to emit `SaveWorldEvent` + set `PendingExitAction::ShipEditor`, transitioning only after `WorldSaved` completes.
- [ ] 3.2 Update `flush_pending_exit_action` to run on `WorldSaved` (and recover cleanly on `SaveIoFailed` with notice).
- [ ] 3.3 Route Home Save Slot Picker listing (`scene/home/slots.rs`) through `IoCommand::ListSlots` so opening Worlds never blocks UI rendering.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests covering `IoWorker` FIFO ordering, `SaveState` busy rejection, stale-sequence cancellation, single-pass `ContinueWorldEvent` from fresh Home state, and Pause → Ship Editor capture-before-despawn ordering.
- [ ] 4.2 Update `docs/save.md` and `docs/scenes.md` (spec only, no notes) documenting `IoWorker`, `SaveState`, and `PendingExitAction::{None, Home, ShipEditor}`.
