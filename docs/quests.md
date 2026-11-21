# Quests Spec

## Purpose

Quests replace the arena mission loop. Gameplay continues after the last enemy is destroyed;
player defeat respawns the player instead of ending the session. Quests are persisted inside the
active save slot and are the only source of persistent player-facing objectives.

## Quest Identity and Kinds

`QuestId` is `Authored(String)` or `Chunk(IVec2)`. The two kinds share one `QuestDef`, one
progression engine, and one `QuestTracker` resource.

- **Authored quests** come from `core/assets/config/quest/quests.ron`. They are auto-accepted
  (`QuestAcceptance::Auto`), always authored, and chain via `requires` + `requires_mode`.
- **Generated quests** derive their identity from a discovered chunk coordinate. They carry a
  `QuestOffer { position, radius }`, use objective values from `generation.ron`, and may be
  auto-accepted or manual.

`QuestAcceptance::Manual` is rejected on authored quests. A manual quest without an `offer` is
rejected. Objective progress is keyed by each objective's stable `id`, so reordering objectives
under hot reload cannot misattribute progress.

## Prerequisite Modes

`QuestPrereqMode::{Completed, Started}` controls what a `requires` entry means:

- `Completed` (default): the quest unlocks when every prerequisite is `Completed`.
- `Started`: the quest unlocks when every prerequisite is `Active` or `Completed`. Tutorial
  chains use this so successors activate as soon as a predecessor starts, without blocking play.

## Definition Assets

| File | Asset | Purpose |
| --- | --- | --- |
| `config/quest/quests.ron` | `QuestCatalogDef` | Authored quest catalog (`ValidCatalog`), validated at parse time |
| `config/quest/generation.ron` | `QuestGenConfig` | Generated quest tuning: caps, radius, objective `OrderedRange`s, `NonEmpty` item pool |
| `config/base/skirmish.ron` | `SkirmishDef` | QuickBattle enemy spawn list (unsaved skirmish only) |

Both quest assets use `RonAssetLoader` and an `AssetEvent` hot-reload bridge. Hot reload replaces
authored catalog entries only; generated catalog entries are preserved. A startup initialization
system applies the assets once config is ready so initial loads are never missed.

Validation rejects: duplicate/empty quest ids, empty objectives, non-positive counts/radii,
unknown prerequisites, prerequisite cycles, authored `Manual` quests, and manual quests without
an `offer`.

## Objectives

- `DestroyEnemies { id, count }`: advances on `ShipDestroyed` for a non-player team.
- `ReachPosition { id, position, radius }`: completes when any live player-team ship is within
  `radius` of `position`.
- `CollectItem { id, item, count }`: advances from the cumulative `ItemCollected` message emitted
  by `collect_drop` with the actual collected amount. Progress never regresses when cargo is spent
  or a ship is destroyed.

All objectives advance only when `WorldContext::is_save_slot()` is true. Completing every objective
transitions the quest to `Completed` and emits `QuestCompleted`.

## Procedural Generation

`core/src/world/event.rs` defines `SectorDiscovered { position, chunk }`. The generation system
consumes it, derives `QuestId::Chunk`, skips IDs already present, enforces
`max_active_generated`, and uses the persisted `GameRng` for objective values only.

In v1 there is no emitter for `SectorDiscovered`: generation is schema-complete but dormant until
`world-chunk-streaming` emits sector discoveries. There is no initial generated batch. Generated
quests created with `Auto` acceptance activate immediately; `Manual` quests enter `Offered`.

Tunable numeric ranges (counts, radii, caps, item pool) live in `generation.ron`. The number of
objective kinds and the number of objectives per quest are structural v1 constants in
`generator.rs`: adding a new `QuestObjectiveDef` variant requires code, so exposing their selection
as data provides no benefit yet.

## Notification Queue and Acceptance

A manual quest is offered only while a live player ship is inside its `offer.radius`:

- Entering range shows a small card in the right-center notification queue.
- Leaving range hides the card but keeps the quest `Offered`; re-entering re-offers it.
- Clicking the card opens a non-blocking dialog (no `PauseState` transition) with **Accept** and
  **Close**. Accept moves the quest to `Active`. Close leaves it `Offered`.
- `Offered` quests are never persisted.

The compact active-quest tracker lists `Active` quests and their live objective counters.

## Persistence

`QuestRecord { def, progress }` is encoded with the shared `encode()` helper into the existing
`quests(quest_id, status, record_blob)` table (schema v3; `status` is an integer), committed in the same `BEGIN IMMEDIATE`
transaction as `world_meta` and `active_state`. `Offered` quests are never written.

On load: `QuestId::Authored` uses the current RON definition (falling back to the saved definition
if the id was removed from RON); `QuestId::Chunk` uses the saved definition verbatim.

## Session Lifecycle

Play time is owned by `SessionClock`, not by the removed mission system. Saving writes
`clock.elapsed(now)` to `WorldMeta.play_time_secs`; loading resumes the clock from the saved value.
Auto-save runs only in `SceneState::Game` while no pause modal is open.

## Player Defeat

`respawn_player_on_defeat` runs in `FixedPostUpdate` / `SimulationSet::Death` after
`apply_body_mutations`, gated only on `SceneState::Game`. When the last player-team ship is
destroyed, a starter player ship spawns at `(0, 0)` with `ControlledShip` and `CameraTarget`
reattached. World entities, chunks, and quest progress are preserved. This applies in save slots
and `QuickBattle`; the respawn collision at the origin is accepted for v1.
