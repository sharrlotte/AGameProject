## Context

`core/src/mission/` currently drives a single arena: `MissionDef` (`quick-battle.ron`), `BattlePhase::{Idle, Loading, Active, Ended}`, and `BattleOutcome::{Victory, Defeat}` when either side's live ship count reaches zero. With `world-save-slots` providing persistent per-slot SQLite saves, that loop no longer fits. The save system itself is coupled to the mission module: `save/world/mod.rs` gates auto-save on `BattlePhase::Active`, and `save/world/system.rs` derives `play_time_secs` from `BattleResult.started_at`. Removing the mission module therefore requires replacing those behaviors, not just deleting files.

The quest system must serve two kinds of content: authored tutorial quests (stable, hot-reloadable, sequential) and procedurally generated quests (dynamic, chunk-scoped, unbounded by authored data). Both share one progression engine.

## Goals / Non-Goals

**Goals:**

- Remove `core/src/mission/` and all arena outcome/overlay behavior while keeping `QuickBattle` functional.
- Replace arena defeat with continuous open-world respawn in both save slots and `QuickBattle`.
- Build a unified, data-driven quest plugin for authored tutorial quests and chunk-generated quests with one progression engine.
- Make tutorial chains unlock successors on *start* (not completion) without blocking gameplay.
- Give generated quests deterministic chunk-derived IDs, bounded active counts, and a range-gated notification flow.
- Persist only accepted quests, atomically with the rest of the save slot.
- Keep skirmish activity from contaminating saved quest progress.

**Non-Goals:**

- Emitting `SectorDiscovered` itself. The event is defined here in `core/src/world/event.rs`; `world-chunk-streaming` emits it when implemented. v1 therefore has no active generation source.
- An initial batch of generated quests.
- NPC dialogue, stations, quest pinning, or abandoning quests. Manual offers have Accept + Close only.
- New SQLite schema. The `quests` table from `world-save-slots` is reused as-is.

## Decisions

### 1. Session lifecycle replaces mission state

`SessionClock` owns accumulated play time and a start instant. `handle_save_world_event` reads `clock.elapsed(time.elapsed_secs())`; `spawn_loaded_world` calls `clock.resume(now, play_time_secs)`. Auto-save is gated on `in_state(SceneState::Game)` and `in_state(PauseState::Playing)` instead of `BattlePhase::Active`. `BattleMode` and `BattleResult.mission_name` are deleted. This gives timekeeping one owner and avoids a save-schema migration.

*Alternatives considered:* persisting the origin timestamp in `WorldMeta` (requires a v1→v2 DB migration); re-homing `BattlePhase`/`BattleResult` as `GamePhase`/`SessionStats` (smaller diff but keeps arena semantics).

### 2. Respawn is mode-independent; quest tracking is not

`respawn_player_on_defeat` runs in `FixedPostUpdate`/`SimulationSet::Death` after `apply_body_mutations`, gated only on `SceneState::Game`. Quest activation and objective systems are additionally gated on `WorldSession::SaveSlot`. A shared `save_slot_only` run condition prevents both activation and `DestroyEnemies` counting during skirmishes, while death in a skirmish still respawns the player so play can continue.

*Alternatives considered:* gating respawn on `SaveSlot` too (left skirmishes unplayable with no end overlay); reviving `BattleMode` as the gate (two sources of truth alongside `WorldSession`).

### 3. Unified quest model with typed identity

```rust
pub enum QuestId { Authored(String), Chunk(IVec2) }
pub enum QuestAcceptance { Auto, Manual }
pub enum QuestPrereqMode { Completed, Started }
pub enum QuestStatus { Offered, Active, Completed }
pub struct QuestOffer { pub position: Vec2, pub radius: f32 }
pub struct QuestDef { /* id, title, description, requires, requires_mode, acceptance, offer, objectives */ }
pub struct QuestProgress { pub quest_id: QuestId, pub status: QuestStatus,
                           pub objective_counts: HashMap<String, u32> }
pub struct QuestRecord { pub def: QuestDef, pub progress: QuestProgress }
```

One `QuestDef` and one progression path serve both kinds; `QuestId` namespaces authored and generated quests so they cannot collide. Authored quests are always `Auto` (validation rejects `Manual` on `QuestId::Authored`). Generated quests derive IDs from chunk coordinates, so discovery dedupes by a simple existence check in catalog/tracker.

*Alternatives considered:* separate `AuthoredProvider`/`GeneratedProvider` types behind a shared tracker; hashing chunk coordinates into a `u64`; splitting generated quests into a separate struct.

### 4. Prerequisite mode supports tutorial chains

`requires_mode: Started` treats a prerequisite as met once it is `Active` or `Completed`, so tutorial quest N+1 activates when N is accepted while N is still in progress. `Completed` (the default) preserves completion-gated unlocking for general quests. This makes "auto-accepted before previous quest is done, but not blocking anything" explicit rather than implicit.

*Alternatives considered:* globally redefining `requires` as start-based (would unlock non-tutorial quests too early); a second `follows` field (two concepts to author and validate).

### 5. Definitions, validation, and hot reload

A single `QuestCatalogDef` asset loads from `core/assets/config/quest/quests.ron` via the existing `RonAssetLoader` + `AssetEvent` bridge pattern. Validation rejects duplicate/empty IDs, empty objectives, non-positive counts/radii, unknown prerequisite IDs, prerequisite cycles, `Manual` authored quests, and `Manual` quests without an `offer`. Hot reload replaces only `Authored` catalog entries; generated entries are preserved.

### 6. Objective progress is event-driven and keyed by stable IDs

`objective_counts` is a `HashMap<String, u32>` keyed by each objective's stable `id` (authored in RON, assigned by the generator), so reordering objectives under hot reload cannot misattribute progress. `DestroyEnemies` increments on `ShipDestroyed` for non-player teams; `ReachPosition` completes when any live player-team core is within radius; `CollectItem` increments from a new `ItemCollected` message emitted by `collect_drop` with the actual collected delta, making progress cumulative and immune to spending or respawn. Completion transitions to `Completed` and emits `QuestCompleted`.

### 7. Procedural generation is deterministic, bounded, and dormant in v1

`generation.ron` supplies `max_active_generated`, `notification_radius`, `manual_acceptance_every`, and objective value ranges. `GameRng` seeds objective *values* only; identity comes from `QuestId::Chunk`. A `SectorDiscovered { position, chunk }` message is defined in `core/src/world/event.rs` so both changes depend on a lower layer rather than on each other. The generation system consumes that message, skips IDs already present, caps active generated quests, and does nothing when no message is emitted. No initial batch and no v1 emitter.

*Alternatives considered:* quest-owned distance emitter (would duplicate what chunk streaming will do); consuming `ChunkLoaded` directly (mutually blocking with `world-chunk-streaming`).

### 8. Manual acceptance uses range-gated notification cards

`QuestOffer { position, radius }` anchors a generated quest to the world. While a live player ship is within radius, a small card appears in a right-center notification queue; leaving range hides the card but keeps the quest `Offered`. Clicking a card reveals a non-blocking dialog with Accept and Close. Accept transitions to `Active`; Close leaves it `Offered`, and revisiting the range re-offers it. Offered quests are never persisted. There is no decline marker.

*Alternatives considered:* modal popup with `PauseState`; single offer replaced by newest; persisting a declined-chunk set.

### 9. Persistence and definition authority

`QuestRecord` (full definition + progress) is encoded with the existing `encode()` helper into `quests.progress_blob`; the existing `quest_id`/`status` columns remain. The save plugin passes `QuestTracker` into `save_world` so quest rows commit in the same `BEGIN IMMEDIATE` transaction as world state, and restores them in `load_world_data`/`apply_world_data`. On load, `QuestId::Authored` uses the current RON definition (falling back to the saved one if the ID was removed from RON); `QuestId::Chunk` uses the saved definition verbatim.

*Alternatives considered:* storing quests in `ActiveStateSnapshot` as another blob (bypasses the reserved table); saved-blob-wins for all (RON tuning never reaches existing saves).

### 10. HUD layout

A compact active-quest tracker lists `Active` quests and their objective counters, updating on `resource_changed::<QuestTracker>`. The manual-offer notification queue is a separate right-center panel. Systems write the tracker only when progress actually changes to avoid per-frame UI churn.

### 11. Module layout

```
core/src/quest/
├── mod.rs          # QuestPlugin, QuestCompleted messages, run conditions
├── component.rs    # QuestTracker, QuestProgress, QuestStatus, LastExploredAt?, QuestHudRoot
├── def.rs          # QuestId, QuestDef, objectives, validation, RON loader + bridge
├── generator.rs    # build(chunk, config, rng) -> QuestDef
├── system.rs       # activation, objective updates, respawn, generation
└── hud.rs          # setup/update tracker, notification queue, offer dialog
```

All plugins in `mod.rs`, types in `component.rs`/`def.rs`, systems in `system.rs` or `hud.rs`, matching the project's `save/world` and `control` layout.

## Risks / Trade-offs

- **No active generation in v1** → state this explicitly in docs; do not claim procedural quests appear until `world-chunk-streaming` lands.
- **Save-schema drift between RON and saves** → stable objective IDs plus definition authority rules; unknown authored IDs fall back to saved definitions.
- **Hot reload reordering** → keyed objective counts instead of positional vectors.
- **Respawn collision at `(0, 0)`** → accepted for v1; a future `RespawnPoint(Vec2)` resource can supply a dynamic point.
- **Skirmish contamination** → `save_slot_only` run condition covers all quest systems; covered by a dedicated test.
- **Notification spam** → range gating plus ID existence check; leaving range only hides the card.
- **Circular change dependency** → avoided by defining `SectorDiscovered` in `core/src/world/event.rs`.

## Migration Plan

- The `quests` table already exists in all slot databases (created by `world-save-slots`), so no schema change is required; the table is currently empty, so there is no quest data to migrate.
- Delete `core/src/mission/` and its asset; replace all mission-type consumers in `save/world/` with `SessionClock` and scene-state gates in the same change to keep the build green.
- Replace `docs/mission.md` with `docs/quests.md` and update the affected docs in the same change.

## Open Questions

- None blocking. Coordination item for a separate change: `world-chunk-streaming` must adopt and emit `SectorDiscovered` from `core/src/world/event.rs`.
