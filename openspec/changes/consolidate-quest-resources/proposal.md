## Why

The quest module currently defines 7 separate resources (`QuestCatalogValue`, `QuestGenConfigValue`, `QuestHandles`, `QuestAssetsReady`, `QuestTracker`, `QuestOfferQueue`, `VisibleOffers`), causing three concrete bugs and unnecessary runtime overhead:
1. `VisibleOffers.0` and `QuestOfferQueue.pending` store identical clones (`quest/system.rs:309-310`), and `queue.pending` is never read anywhere.
2. `QuestAssetsReady` exists solely to work around a startup `AssetEvent` race, forcing `load_initial_assets` to poll every `Update` frame forever.
3. `QuestCatalogValue` mixes global RON-authored quests with per-slot generated `QuestId::Chunk` defs (`quest/system.rs:261`). On slot reload, `spawn_loaded_world` restores records only into `QuestTracker`, so `update_quest_objectives` (`catalog.0.get(&id)` at `quest/system.rs:164`) **silently stops progressing every reloaded generated quest** (and removed authored quest), while `handle_create_slot` never clears prior slot entries, leaking generated quests across save slots.
4. `sync_offer_cards` and `sync_offer_dialog` (`quest/hud.rs:123,162`) cache `Local` state that goes stale across `SceneState::Game` re-entry, and `activate_quests` mutates `ResMut<QuestTracker>` every 60 Hz tick even when nothing changes.

## What Changes

- Consolidate the 7 quest resources into **3 focused resources grouped cleanly by lifetime and mutability**:
  1. `QuestConfig` (authored RON catalog + generation config, loaded and validated inside `ConfigPlugin`'s boot gate — eliminating `QuestCatalogValue`, `QuestGenConfigValue`, `QuestHandles`, `QuestAssetsReady`, and `load_initial_assets`).
  2. `QuestTracker` (per-slot persistent runtime state; sole owner of generated `QuestDef`s inside `QuestRecord`; read `record.def` directly during objective updates so reloaded generated quests and removed-authored quests always progress; reset cleanly on slot create/load/exit; avoid spurious `DerefMut` change notifications).
  3. `QuestOfferUi { visible: Vec<QuestId>, open: Option<QuestId> }` (merging `VisibleOffers` + `QuestOfferQueue`, removing dead `pending`, and replacing stale `Local` HUD caches with `resource_changed::<QuestOfferUi>` + `OnEnter` refresh).

## Capabilities

### New Capabilities
- `quest-resource-model`: Three-resource ownership split (`QuestConfig`, `QuestTracker`, `QuestOfferUi`) separating read-only boot-gated RON config, per-slot persistent runtime state, and transient HUD UI projection.

### Modified Capabilities
- `open-world-quests`: Update catalog loading to populate `QuestConfig` inside `ConfigPlugin`'s boot gate, keep `QuestConfig` authored-only (never mixed with per-slot generated quests), and require objective progression to read `record.def` from `QuestTracker` so reloaded generated and removed-authored quests progress reliably.

## Non-Goals

- Splitting monolithic objective handlers into separate event listeners (handled by `event-driven-quest-listeners`).
- Replacing `context.is_save_slot()` with `GameMode` / `CampaignGame` states (handled by `game-mode-state`).
- Merging all other game configs (`ai`, `minimap`, `skirmish`) into `GameConfig` / `game.ron` (handled by `unify-config-and-settings`, which embeds `QuestConfig` cleanly).

## Impact

- `core/src/quest/` (`mod.rs`, `component.rs`, `system.rs`, `hud.rs`, `lib.rs`).
- `core/src/config/mod.rs` (load & validate quest RON assets during `ConfigState::Loading -> Ready`).
- `core/src/save/world/system.rs` (reset `QuestTracker` and `QuestOfferUi` cleanly on slot/session transitions).
- `docs/quests.md` and `docs/architecture.md` (spec-only updates).
