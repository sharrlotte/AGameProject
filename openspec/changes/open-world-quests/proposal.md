## Why

The game is moving from isolated arena missions to a persistent open world. The current arena loop ends when the last enemy dies, wipes the session on player death, and offers no persistent player-facing goals. With `world-save-slots` providing per-slot SQLite saves, quests, progression, and respawn can live inside the active save slot instead of a mission state machine.

## What Changes

- **BREAKING**: Remove `core/src/mission/` (`MissionPlugin`, `MissionDef`, `BattlePhase`, `BattleOutcome`, `BattleResult`, `BattleRestart`, `quick-battle.ron` mission flow, and the Victory/Defeat end overlay). Migrate the initial fleet/enemy spawn helper into `core/src/scene/game/skirmish.rs`.
- **BREAKING**: Replace arena outcome handling with open-world respawn. When the last player-team ship is destroyed, spawn a fresh starter player ship at the configured respawn point (default `(0, 0)`), re-attach `ControlledShip` and `CameraTarget`, and preserve all world entities, chunks, and quest progress. This applies in both save slots and `QuickBattle`. Enemy destruction never ends the game.
- **BREAKING**: Replace mission-derived session state in the save system with a dedicated `SessionClock` resource; gate auto-save on `SceneState::Game` + `PauseState::Playing`. Remove `BattleMode` and `BattleResult.mission_name`.
- Introduce a data-driven `core/src/quest/` plugin supporting **two quest kinds**:
  - **Authored tutorial quests** from a hot-reloadable `core/assets/config/quest/quests.ron` catalog. They are auto-accepted and chain by `requires_mode: Started`, so a successor activates as soon as its predecessor starts and multiple tutorial quests can be active at once without blocking anything.
  - **Procedurally generated quests** whose IDs derive deterministically from discovered chunk coordinates (`QuestId::Chunk(IVec2)`), with tuning loaded from `core/assets/config/quest/generation.ron`.
- Quest model: `QuestId::{Authored(String), Chunk(IVec2)}`, `QuestAcceptance::{Auto, Manual}`, `QuestPrereqMode::{Completed, Started}`, `QuestOffer { position, radius }`, `QuestStatus::{Offered, Active, Completed}`, and objective progress keyed by stable objective IDs (safe under hot reload).
- Event-driven objective progression: `DestroyEnemies` from `ShipDestroyed`, `ReachPosition` from live player-ship positions, `CollectItem` from a new cumulative `ItemCollected` message.
- Procedural generation is **schema-complete but dormant in v1**: it runs only when a shared `SectorDiscovered` message is emitted. `core/src/world/event.rs` defines that message so `world-chunk-streaming` can emit it later without a circular change dependency. No initial generated batch.
- Manual quests use a right-center notification queue: a small card appears while the player is within `QuestOffer.radius`, hides when out of range (the offer itself is kept), and clicking reveals a non-blocking dialog with Accept + Close. Close leaves the quest `Offered`; revisiting re-offers it. Authored quests cannot be `Manual`.
- Persist active/completed quests and their full definitions in the active save slot's existing SQLite `quests` table, atomically with world state. `Offered` quests are never saved. On load, RON definitions win for authored IDs and saved definitions win for generated IDs.
- Compact HUD quest tracker for active quests, plus the separate right-center manual-offer notification queue.
- Keep `QuickBattle` as a functional unsaved skirmish. Quest activation and progression are gated on `WorldSession::SaveSlot`, so skirmish kills cannot alter saved quest progress.
- Update `/docs/quests.md` (new), `/docs/mission.md` (removed), `/docs/scenes.md`, `/docs/ui.md`, `/docs/save.md`, and `/docs/config.md`.

## Capabilities

### New Capabilities
- `open-world-quests`: Authored and chunk-generated quest definitions, typed quest IDs, prerequisite modes, acceptance modes, stable-ID objective tracking, event-driven progression, chunk-scoped generation, range-gated offer notifications, SQLite `quests` persistence, compact HUD tracker, and session lifecycle replacement.

### Modified Capabilities
- `battle-outcome`: Replace arena Victory/Defeat transitions with continuous play and origin respawn on last-player-ship destruction in both save slots and `QuickBattle`; remove mission-scoped scene logic and the end overlay.

## Impact

- **Systems & Modules**:
  - Delete `core/src/mission/`; move the skirmish spawn helper to `core/src/scene/game/skirmish.rs`.
  - New `core/src/quest/` plugin: `mod.rs`, `component.rs`, `def.rs`, `generator.rs`, `system.rs`, `hud.rs`.
  - New shared event module `core/src/world/event.rs` (`SectorDiscovered`).
  - `core/src/save/world/`: extend `WorldStore` and both implementations with `quests` CRUD; add `SessionClock`-based play-time and quest capture/restore to `handle_save_world_event` / `load_world_data` / `apply_world_data` / `spawn_loaded_world`; update the auto-save run condition.
  - `core/src/content/item/`: new `ItemCollected` message emitted by `collect_drop`.
  - `core/src/scene/game/`: mount the quest HUD and notification queue with `DespawnOnExit(SceneState::Game)`.
- **Assets**: `core/assets/config/quest/quests.ron`, `core/assets/config/quest/generation.ron`; remove `core/assets/config/mission/quick-battle.ron`.
- **Dependencies**: Uses the existing `quests` table from `world-save-slots` (no schema migration). `world-chunk-streaming` must emit `SectorDiscovered` when implemented.
- **Documentation**: New `docs/quests.md`; update `docs/scenes.md`, `docs/ui.md`, `docs/save.md`, `docs/config.md`; remove `docs/mission.md`.
