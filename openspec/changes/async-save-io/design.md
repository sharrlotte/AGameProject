## Context

Currently `handle_save_world_event` (`core/src/save/world/system.rs:38-94`) runs inside `Update` taking `&World`: it runs 3 full `world.iter_entities()` scans, `SELECT`s previous `world_meta`, runs `postcard` + `lz4_flex` compression, executes a `BEGIN IMMEDIATE` transaction, and runs `PRAGMA wal_checkpoint(TRUNCATE)` — all blocking the render/simulation thread. `handle_load_slot`, `handle_create_slot`, `handle_continue_world` (which also reads and decodes the full world twice!), and `SqliteWorldStore::drop` similarly block the main thread.

Furthermore:
- `scene/pause/mod.rs:87-104` emits `SaveWorldEvent` and immediately calls `scene.set(SceneState::ShipEditor)` inside a `PreUpdate` click observer, so `StateTransition` fires `OnExit(SceneState::Game)` and despawns every gameplay entity *before* `handle_save_world_event` executes in `Update`.
- `save/world/system.rs:245` calls `context.store().list_slots()` when `WorldContext` defaults to `QuickBattle { store: InMemoryWorldStore }`, returning `[]` on fresh launch instead of reading `<data_dir>/saves/*.db`.

## Goals / Non-Goals

**Goals:**
- Move all encoding, `lz4_flex` compression, SQLite queries/transactions, file opens/creates, and WAL checkpoints onto a dedicated background I/O worker thread.
- Guarantee strict FIFO ordering across autosaves, exit saves, design saves, and future chunk writes without `Mutex` contention.
- Keep main-thread capture fast (extract owned structs from `&World`, hand off immediately).
- Prevent overlapping saves/loads via `SaveState::{Idle, Saving, Loading}` and emit explicit completion messages (`WorldSaved`, `WorldLoaded`, `IoFailed`).
- Fix the Pause → Ship Editor save race and Home `ContinueWorldEvent` bug.

**Non-Goals:**
- Multi-threaded parallel SQLite writers (SQLite supports one writer at a time; a single writer thread matches SQLite's concurrency model cleanly).
- Changing snapshot field contents or DB schema version (covered by other proposals).

## Decisions

### 1. Dedicated Single I/O Worker Thread + Bounded Channel (`IoWorker`)
- **Choice**: Spawn one named background thread (`"world-io-worker"`) initialized inside `WorldSavePlugin`, communicating via a bounded `std::sync::mpsc::sync_channel<IoCommand>(BOUNDED_CAPACITY)` and an unbounded result channel `std::sync::mpsc::channel<IoResult>()` (draining results every `Update`).
  ```rust
  pub enum IoCommand {
      AttachStore(Box<dyn WorldStore>),
      DetachToMemory,
      SaveWorld { seq: u64, captured: CapturedWorld, flush: bool },
      OpenAndLoad { seq: u64, path: PathBuf, slot_id: Uuid, registry_keys: HashSet<String> },
      ResolveAndContinue { seq: u64, active_slot: Option<Uuid>, saves_dir: PathBuf },
      CreateSlot { seq: u64, slot_id: Uuid, path: PathBuf, name: String, seed: u64, designs: Vec<ShipDesign> },
      SaveDesign { seq: u64, design: ShipDesign },
      ListSlots { seq: u64, saves_dir: PathBuf },
      Shutdown,
  }
  ```
- **Rationale**:
  - Owning `Box<dyn WorldStore>` exclusively on the worker thread removes lock contention between autosave, exit flush, design writes, and chunk streaming.
  - Strict FIFO ordering guarantees that a `SaveDesign` followed by `SaveWorld` followed by `Flush` executes in exact causal order.
  - A bounded command channel (e.g., capacity 32) provides backpressure without silently growing memory.
  - `Drop` on `IoWorker` sends `IoCommand::Shutdown` and joins the thread so the final WAL checkpoint finishes cleanly on application exit.
- **Alternative Considered**: *`AsyncComputeTaskPool` + `Arc<dyn WorldStore>`* — concurrent tasks can execute out-of-order and contend on `Mutex<Connection>`, and shutting down cleanly with an in-flight WAL checkpoint requires manual task joining anyway.

### 2. Owned Main-Thread Capture + `SaveState` Machine
- **Choice**:
  - Introduce `#[derive(Resource, Default, PartialEq, Eq)] pub enum SaveState { #[default] Idle, Saving { seq: u64 }, Loading { seq: u64 } }`.
  - When `SaveWorldEvent` fires while `SaveState::Idle`, the main thread runs one lightweight single-pass capture of owned Rust structs (`CapturedWorld { play_time_secs, seed, camera, control, bodies, drops, intel, quests }`) — NO `postcard`, NO `lz4_flex`, NO SQLite — sets `SaveState::Saving { seq }`, and enqueues `IoCommand::SaveWorld`.
  - Autosave (`tick_auto_save`) skips/ignores while `SaveState != Idle`. Explicit exit saves (`PendingExitAction::{Home, ShipEditor}`) either start immediately if `Idle`, or coalesce/wait for the in-flight save to finish and flush before transitioning.
  - `poll_io_results` runs in `Update` (`SimulationSet::Present`), drains `IoResult`, resets `SaveState::Idle`, writes `WorldSaved` / `WorldLoaded` / `AppNotice`, and triggers `PendingExitAction` once the exit save succeeds.
  - On `SceneState` exit during an in-flight `Loading` job, stale sequence numbers (`result.seq != active_seq`) are discarded cleanly (cancellation by generation token).

### 3. Fixing Pause → Ship Editor Race & Home Continue Bug
- **Choice**:
  - Expand `PendingExitAction` to `{ None, Home, ShipEditor }` (still no app-exit variant, preserving `scene-lifecycle`). `on_ship_editor` in `scene/pause/mod.rs` sets `PendingExitAction::ShipEditor` and emits `SaveWorldEvent` when in a save slot instead of calling `scene.set(SceneState::ShipEditor)` immediately.
  - `handle_continue_world` always resolves disk slots from `saves_dir()` when no active save slot is loaded (fixing `InMemoryWorldStore::list_slots()` returning `[]`), opens the slot once on the worker, reads `meta`, `active_state`, `designs`, and `quests` once, sends the decoded `WorldLoadData` back via `IoResult::WorldLoaded`, and validates against `BlockRegistry` on the main thread before transitioning to `SceneState::Game` — eliminating the duplicate `load_world_data` read.

## Risks / Trade-offs

- **Main-thread capture cost on huge worlds** → Single-pass entity iteration collecting `ShipSnapshot`/`DebrisSnapshot` takes <1ms even for hundreds of entities; encoding + lz4 + SQLite + fsync (the real 10–80ms hitch) moves completely off-thread.
- **Validation needs `BlockRegistry`** → Keep `BlockRegistry` on the main thread: worker decodes `WorldLoadData` + `Vec<ShipDesign>` off-thread, main thread runs `validate` + `validate_references` before spawning entities (fast in-memory check, keeps `BlockRegistry` uncloned).
- **Worker thread panic or channel disconnect** → `try_send` / `try_recv` failures transition `SaveState::Idle`, log `error!`, and show an `AppNotice` — never `.unwrap()` or fail silently.
- **Autosave flush frequency** → Autosaves commit the SQLite WAL transaction without calling `PRAGMA wal_checkpoint(TRUNCATE)` every 30s (`flush: false`); full `flush: true` runs on scene exit (`Home`, `ShipEditor`) and worker shutdown, further cutting disk stalls.
