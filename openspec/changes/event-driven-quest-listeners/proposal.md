## Why

`update_quest_objectives` (`core/src/quest/system.rs:127-219`) is a 93-line monolith running every 60 Hz `FixedPostUpdate` tick that mixes three unrelated responsibilities — reading `ShipDestroyed`, reading `ItemCollected`, and polling every player ship's `Transform` for `ReachPosition` — and then re-running `activate_available`. Meanwhile, `activate_quests` (`quest/system.rs:123-125`) ALSO polls the entire quest catalog at 60 Hz every tick, `generate_from_discovery` runs even when zero `SectorDiscovered` messages exist, and `sync_offer_visibility` scans all ships every render frame even when zero `Offered` quests exist.

## What Changes

- Split `update_quest_objectives` into dedicated, single-responsibility message listener systems gated by Bevy's `.run_if(on_message::<T>)`:
  - `on_ship_destroyed` (`.run_if(on_message::<ShipDestroyed>)`) advancing `DestroyEnemies` objectives.
  - `on_item_collected` (`.run_if(on_message::<ItemCollected>)`) advancing `CollectItem` objectives.
  - `check_reach_position` (throttled via `on_timer` and skipped whenever no active `ReachPosition` objective exists) advancing `ReachPosition` objectives.
- Gate `generate_from_discovery` with `.run_if(on_message::<SectorDiscovered>)` and `respawn_player_on_defeat` with `.run_if(on_message::<ShipDestroyed>)`.
- Emit `QuestActivated { quest_id }` when quests activate or are manually accepted, and replace 60 Hz polling `activate_quests` with a reactive system gated on `OnEnter`, `resource_changed` (config), `on_message::<QuestCompleted>`, and `on_message::<QuestActivated>` (supporting `QuestPrereqMode::Started` chains with zero idle polling).
- Extract a shared stateless helper (`advance_matching_objectives`) that only mutates `QuestTracker` when progress actually changes, evaluates completion in one place, and emits `QuestCompleted`.
- Throttle `sync_offer_visibility` and skip immediately when no `QuestStatus::Offered` records exist.

## Capabilities

### New Capabilities
- `quest-event-listeners`: Decoupled per-event quest listener systems (`on_ship_destroyed`, `on_item_collected`, `on_sector_discovered`, throttled `check_reach_position`, and reactive `activate_quests` driven by `QuestCompleted` / `QuestActivated`).

### Modified Capabilities
- `open-world-quests`: Specify separate event-gated listeners for each objective kind and reactive prerequisite activation instead of 60 Hz monolithic polling.

## Non-Goals

- Migrating combat/item/world messages (`ShipDestroyed`, `ItemCollected`, `SectorDiscovered`) from Bevy `Message` to Bevy `Observer` (`On<T>`).
- Adding new objective variants (rewards, timed quests, escort/failure states).

## Impact

- `core/src/quest/` (`mod.rs`, `component.rs`, `lib.rs`, `system.rs`, `hud.rs`).
- `docs/quests.md` and `docs/architecture.md` (spec-only updates).
