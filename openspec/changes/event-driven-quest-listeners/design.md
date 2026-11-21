## Context

Currently in `core/src/quest/mod.rs:58-74` and `system.rs:123-311`:
- `activate_quests` runs unconditionally 60 times/second scanning the entire catalog `HashMap`.
- `update_quest_objectives` runs unconditionally 60 times/second, allocating `Vec<Vec2>` for player positions and iterating every active quest even on ticks with zero kills, zero pickups, and zero `ReachPosition` objectives.
- `generate_from_discovery` and `respawn_player_on_defeat` run every tick without `.run_if(on_message::<...>)`.
- `QuestCompleted` is emitted (`system.rs:214`) but has zero listeners, and manual acceptance in `hud.rs:248` (`on_accept_offer`) never emits an activation message, relying on the next tick's 60 Hz `activate_quests` poll.

## Goals / Non-Goals

**Goals:**
- Separate each objective/event domain into its own small, focused, testable listener system gated by `.run_if(on_message::<T>)`.
- Eliminate all 60 Hz idle polling in `QuestPlugin` (`activate_quests`, `update_quest_objectives`, `generate_from_discovery`, `respawn_player_on_defeat`).
- Keep adding a new event-driven objective kind (e.g., `VisitStation`, `CraftBlock`) a matter of adding one new listener system calling the shared `advance_matching_objectives` helper.

**Non-Goals:**
- Replacing `MessageWriter`/`MessageReader` with `commands.trigger`/`On<T>` observers across combat/item/world plugins.
- Rewriting `QuestTracker` resource layout (handled by `consolidate-quest-resources`).

## Decisions

### 1. Per-Event `MessageReader<T>` Systems + Shared Advancement Helper
- **Choice**: Define a shared pure-logic helper in `core/src/quest/lib.rs`:
  ```rust
  pub fn advance_objectives(
      tracker: &mut QuestTracker,
      mut delta_for: impl FnMut(&QuestObjectiveDef) -> u32,
  ) -> Vec<QuestId>
  ```
  - Scans active records immutably first (or collects `(QuestId, objective_id, new_val)` deltas) so `ResMut<QuestTracker>` is only mutably dereferenced when at least one objective actually increases!
  - Marks newly completed quests as `QuestStatus::Completed` and returns completed `QuestId`s so the caller emits `QuestCompleted` messages cleanly.
  - Register focused listeners in `FixedPostUpdate::Death` (`.after(apply_body_mutations)`):
    - `on_ship_destroyed` (`.run_if(on_message::<ShipDestroyed>)`)
    - `on_item_collected` (`.run_if(on_message::<ItemCollected>)`)
    - `on_sector_discovered` (`.run_if(on_message::<SectorDiscovered>)`)
    - `respawn_player_on_defeat` (`.run_if(on_message::<ShipDestroyed>)`)
  - Register spatial proximity checks (`check_reach_position`) on a throttled timer (e.g., `on_timer(Duration::from_millis(100))`) with an early run condition `has_active_reach_objectives` so zero work runs when no `ReachPosition` quest is active.
- **Rationale**: Matches existing `ShipDestroyed` / `ItemCollected` / `SectorDiscovered` buffered messages, respects deterministic `FixedPostUpdate::Death` set ordering, and makes every listener <30 lines and independently unit-testable.
- **Alternative Considered**: *Bevy 0.20 `Observer`s (`On<T>`)* — requires modifying every combat/item/world emitter to trigger events instead of writing buffered messages and complicates `FixedPostUpdate::Death` chaining.

### 2. Reactive `activate_quests` via `QuestCompleted` + `QuestActivated`
- **Choice**:
  - Add `#[derive(Message)] pub struct QuestActivated { pub quest_id: QuestId }`.
  - `activate_quests` runs:
    1. Once on entering active campaign gameplay (after world load/create), AND
    2. In `FixedPostUpdate::Death` (chained `.after((on_ship_destroyed, on_item_collected, check_reach_position))`) gated by `.run_if(on_message::<QuestCompleted>.or(on_message::<QuestActivated>).or(resource_changed::<QuestCatalogValue>))` (or `QuestConfig` once consolidated).
  - Inside `activate_available`, loop to fixed-point so a chain of `QuestPrereqMode::Started` tutorial quests (A → B → C) unlocks cleanly in one pass without waiting multiple ticks, emitting `QuestActivated` for every newly activated quest.
  - `on_accept_offer` in `hud.rs` also emits `QuestActivated` when a manual quest is accepted.

### 3. Throttled & Guarded `sync_offer_visibility`
- **Choice**: Gate `sync_offer_visibility` with a run condition `has_offered_quests` (plus `on_timer(Duration::from_millis(100))` or player movement check), clearing UI once when no offers exist instead of scanning all player ships every render frame.

## Risks / Trade-offs

- **Deterministic completion order** → `advance_objectives` collects active IDs sorted by `QuestId::key()` before emitting `QuestCompleted`, removing `HashMap` iteration randomness.
- **Multi-listener ordering in one tick** → Chaining `(on_ship_destroyed, on_item_collected, check_reach_position) -> activate_quests` in `FixedPostUpdate::Death` guarantees that if a kill and an item pickup complete a quest in the same tick, dependent quests unlock before the tick ends.
