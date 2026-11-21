## Context

Inspecting `core/src/quest/component.rs:179-243`, `system.rs:55-311`, and `hud.rs:85-225` revealed why the 7 quest resources feel fragmented and where they actually break:
- `QuestHandles` + `QuestAssetsReady` + `load_initial_assets` only exist because quest RON assets were loaded outside `ConfigPlugin`'s `ConfigState::Loading -> Ready` boot gate.
- `QuestCatalogValue` is mutated during gameplay (`quest/system.rs:261`) to store generated `QuestId::Chunk` quests alongside RON-authored quests. Because `QuestCatalogValue` is neither persisted nor cleared on slot switch, reloaded generated quests disappear from `catalog.0` (breaking `update_quest_objectives:164`) while old slot quests bleed into newly created slots.
- `VisibleOffers` and `QuestOfferQueue` duplicate the exact same `Vec<QuestId>` (`queue.pending` is never read).
- `sync_offer_cards` and `sync_offer_dialog` store `Local<Vec<QuestId>>` / `Local<Option<QuestId>>`, which retain old values after `DespawnOnExit(SceneState::Game)` despawns the UI host and re-entering `Game` spawns an empty host.

## Goals / Non-Goals

**Goals:**
- Reduce 7 quest resources to 3 clean lifetime-scoped resources (`QuestConfig`, `QuestTracker`, `QuestOfferUi`).
- Fix the reloaded-generated-quest progression bug and cross-slot generated-quest leak.
- Preserve Bevy change-detection granularity (`resource_changed::<QuestTracker>`, `resource_changed::<QuestOfferUi>`) and fix HUD re-entry rendering.
- Eliminate `QuestAssetsReady` forever-polling by gating quest RON assets inside `ConfigPlugin`.

**Non-Goals:**
- Merging `QuestConfig`, `QuestTracker`, and `QuestOfferUi` into a single monolithic `QuestState` (doing so would invalidate `resource_changed` on every UI hover/click and serialize unrelated systems).
- Refactoring objective update systems into per-event listeners (handled by `event-driven-quest-listeners`).

## Decisions

### 1. Three Lifetime-Scoped Resources
- **Choice**:
  1. **`QuestConfig`** (global read-only config, owned by `ConfigPlugin` — embeds directly into unified `GameConfig` when `unify-config-and-settings` lands):
     ```rust
     #[derive(Resource, Default, Debug, Clone)]
     pub struct QuestConfig {
         pub authored: HashMap<QuestId, QuestDef>,
         pub generation: QuestGenConfig,
     }
     ```
     Loaded and validated during `ConfigState::Loading -> Ready` alongside `block.ron`/`game.ron`, with hot-reload bridges updating `authored` (and refreshing matching `QuestId::Authored` defs inside `QuestTracker` without touching generated quests!) and `generation`.
  2. **`QuestTracker`** (per-slot persistent runtime state):
     - Sole runtime owner of `records: HashMap<QuestId, QuestRecord>` for both authored and generated quests.
     - Objective updates read `&record.def` directly from `QuestTracker` (never skipping reloaded `QuestId::Chunk` or removed-authored quests!).
     - `activate_available` checks whether any unlockable quests exist before borrowing `&mut tracker` (or only mutates when `newly_activated > 0`), preserving `resource_changed::<QuestTracker>`.
     - Cleared cleanly on `CreateSlot`, `StartQuickBattle`, and exiting a save slot.
  3. **`QuestOfferUi`** (transient HUD projection, replaces `VisibleOffers` + `QuestOfferQueue`):
     ```rust
     #[derive(Resource, Default, Debug, PartialEq, Eq)]
     pub struct QuestOfferUi {
         pub visible: Vec<QuestId>,
         pub open: Option<QuestId>,
     }
     ```
     - `sync_offer_visibility` computes `next` and calls `ui.set_if_neq(...)` (only triggering change detection when visibility/open dialog actually changes!).
     - HUD systems (`sync_offer_cards`, `sync_offer_dialog`) drop `Local` caches and instead trigger on `resource_changed::<QuestOfferUi>` plus `OnEnter(SceneState::Game)` (or `Added<QuestOfferList>` / `Added<QuestOfferDialogHost>`), ensuring cards and dialogs always render freshly on scene re-entry.

## Risks / Trade-offs

- **Authored quest hot-reload synchronization** → When `quests.ron` hot-reloads in dev, `apply_authored_catalog` updates `QuestConfig.authored` and also refreshes `record.def` for any matching `QuestId::Authored` already in `QuestTracker` (preserving `record.progress.objective_counts` keyed by stable objective ID).
- **Coordination with `unify-config-and-settings`** → `QuestConfig` is intentionally structured as a self-contained struct so `unify-config-and-settings` can simply embed `pub quest: QuestConfig` inside `GameConfig` without rewriting quest logic.
