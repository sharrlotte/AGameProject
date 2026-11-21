## 1. Replace 7 Quest Resources with `QuestConfig`, `QuestTracker`, `QuestOfferUi`

- [ ] 1.1 Replace `QuestCatalogValue`, `QuestGenConfigValue`, `QuestHandles`, `QuestAssetsReady`, `QuestOfferQueue`, and `VisibleOffers` in `core/src/quest/component.rs` with `QuestConfig { authored, generation }`, `QuestTracker`, and `QuestOfferUi { visible, open }`.
- [ ] 1.2 Move loading, boot validation (`ConfigState::Loading -> Ready`), and hot-reload bridges for `quests.ron` and `generation.ron` into `ConfigPlugin` (`core/src/config/mod.rs`), removing `load_quest_handles` and `load_initial_assets` from `QuestPlugin`.
- [ ] 1.3 Ensure hot-reloading `quests.ron` refreshes `record.def` for matching `QuestId::Authored` entries in `QuestTracker` without touching generated entries or objective progress.

## 2. Fix Mixed-Ownership Bugs and Change Detection

- [ ] 2.1 Update `generate_from_discovery` (`core/src/quest/system.rs`) so generated `QuestDef`s are stored exclusively in `QuestTracker` (never inserted into `QuestConfig.authored`).
- [ ] 2.2 Update `update_quest_objectives` (`core/src/quest/system.rs`) to read `&record.def` directly from `QuestTracker` (fixing reloaded generated quests and removed-authored quests silently stopping).
- [ ] 2.3 Guard mutations on `ResMut<QuestTracker>` in `activate_quests` and `update_quest_objectives`, and use `set_if_neq` on `ResMut<QuestOfferUi>` in `sync_offer_visibility` so change detection fires only on actual state changes.
- [ ] 2.4 Reset `QuestTracker` and `QuestOfferUi` cleanly on `CreateSlot`, `StartQuickBattle`, and save-slot exit (`core/src/save/world/system.rs`).

## 3. Fix HUD Re-Entry Caching

- [ ] 3.1 Update `core/src/quest/hud.rs` (`update_quest_tracker`, `sync_offer_cards`, `sync_offer_dialog`, button observers) to read `QuestConfig`, `QuestTracker`, and `QuestOfferUi`, removing stale `Local` caches and refreshing on `resource_changed::<QuestOfferUi>` and host spawn (`Added<QuestOfferList>` / `Added<QuestOfferDialogHost>`).

## 4. Verification and Docs

- [ ] 4.1 Add unit tests covering reloaded `QuestId::Chunk` quest progression, cross-slot isolation on `CreateSlot`, `QuestOfferUi` change-detection silence when unchanged, and HUD offer card rendering across `Game` exit/re-entry.
- [ ] 4.2 Update `docs/quests.md` and `docs/architecture.md` (spec only, no notes) documenting `QuestConfig`, `QuestTracker`, and `QuestOfferUi`.
