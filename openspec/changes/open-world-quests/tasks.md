## 1. Foundation and shared event

- [ ] 1.1 Create `core/src/world/mod.rs` and `core/src/world/event.rs` defining `SectorDiscovered { position: Vec2, chunk: IVec2 }`, and register the message in the app
- [ ] 1.2 Create `core/src/quest/mod.rs` with `QuestPlugin`, `QuestCompleted`, and the `save_slot_only` run condition; register `QuestPlugin` in `core/src/main.rs`
- [ ] 1.3 Create `core/src/quest/component.rs` with `QuestId`, `QuestStatus`, `QuestAcceptance`, `QuestPrereqMode`, `QuestOffer`, `QuestProgress`, `QuestTracker`, `QuestRecord`, `QuestOfferQueue`, and `QuestHudRoot`

## 2. Quest definitions and validation

- [ ] 2.1 Create `core/src/quest/def.rs` with `QuestCatalogDef`, `QuestDef`, `QuestObjectiveDef`, `QuestCatalogValue`, and `RonAssetLoader` registration following the `GameConfig`/`AiConfig` bridge pattern
- [ ] 2.2 Implement `validate_catalog` rejecting duplicate/empty IDs, empty objectives, non-positive counts/radii, unknown prerequisites, cycles, `Manual` authored quests, and `Manual` quests without `offer`
- [ ] 2.3 Implement hot-reload bridging that replaces only `Authored` catalog entries and preserves generated entries
- [ ] 2.4 Create `core/assets/config/quest/quests.ron` with authored tutorial quests using `requires_mode: Started`
- [ ] 2.5 Create `core/assets/config/quest/generation.ron` with `max_active_generated`, `notification_radius`, `manual_acceptance_every`, and objective value ranges

## 3. Activation, progression, and generation

- [ ] 3.1 Implement `activate_available_quests` honoring both `QuestPrereqMode` values on catalog load, quest completion, and save load
- [ ] 3.2 Implement objective updates for `DestroyEnemies`, `ReachPosition`, and `CollectItem` keyed by stable objective IDs, with completion emitting `QuestCompleted`
- [ ] 3.3 Add the `ItemCollected` message in `core/src/content/item/` and emit the actual collected delta from `collect_drop` in `core/src/content/ship/system.rs`
- [ ] 3.4 Create `core/src/quest/generator.rs` with `build(chunk, config, rng) -> QuestDef`, deriving `QuestId::Chunk`, assigning stable objective IDs, choosing acceptance from `manual_acceptance_every`, and setting `QuestOffer` from chunk center and `notification_radius`
- [ ] 3.5 Implement the `SectorDiscovered` consumer that dedupes by existing ID and enforces `max_active_generated`
- [ ] 3.6 Implement manual acceptance: range-gated notification queue show/hide, click-to-open non-blocking dialog, Accept transitions to `Active`, Close leaves `Offered`, and revisit re-offers

## 4. Persistence and save integration

- [ ] 4.1 Extend `WorldStore` plus `SqliteWorldStore` and `InMemoryWorldStore` with `load_quests` and `save_quests` using the existing `quests` table and the shared `encode`/`decode` helpers
- [ ] 4.2 Update `handle_save_world_event` to capture `QuestTracker` into `QuestRecord` rows, excluding `Offered` quests, and commit them in the same transaction as world state
- [ ] 4.3 Update `load_world_data` and `apply_world_data` to restore quests, applying authored RON definition authority and generated saved-definition authority
- [ ] 4.4 Ensure reloading a slot replaces `QuestTracker` so no progress leaks between slots

## 5. Session lifecycle and mission removal

- [ ] 5.1 Create the `SessionClock` resource and switch `handle_save_world_event` and `spawn_loaded_world` to use it for `play_time_secs`
- [ ] 5.2 Change the auto-save run condition in `save/world/mod.rs` to `SceneState::Game` + `PauseState::Playing`
- [ ] 5.3 Delete `BattleMode`, its writes in `save/world/system.rs`, and `BattleResult.mission_name`
- [ ] 5.4 Move the initial fleet/enemy spawn helper into `core/src/scene/game/skirmish.rs`
- [ ] 5.5 Replace `check_battle_outcome` with `respawn_player_on_defeat` in `core/src/quest/system.rs`, gated only on `SceneState::Game`, then delete `core/src/mission/` and `core/assets/config/mission/quick-battle.ron`

## 6. HUD

- [ ] 6.1 Create `core/src/quest/hud.rs` with the compact active-quest tracker using `DespawnOnExit(SceneState::Game)` and change-only updates
- [ ] 6.2 Add the right-center notification queue and the manual-offer dialog using the existing `spawn_button` widget
- [ ] 6.3 Mount both HUD panels in `core/src/scene/game/mod.rs`

## 7. Tests

- [ ] 7.1 Unit-test catalog validation including cycles, manual authored rejection, and missing-offer rejection
- [ ] 7.2 Unit-test prerequisite modes (`Started` unlock while predecessor is active; `Completed` waits)
- [ ] 7.3 Unit-test all three objective types, cumulative `CollectItem`, and stable-ID behavior under objective reorder
- [ ] 7.4 Unit-test generation dedupe, cap enforcement, and dormancy without `SectorDiscovered`
- [ ] 7.5 Unit-test notification range show/hide, Accept, Close, and re-offer on revisit
- [ ] 7.6 Unit-test `quests` table persistence round-trip, `Offered` exclusion, and definition authority
- [ ] 7.7 Unit-test respawn on last-player-ship destruction in both save slot and `QuickBattle`
- [ ] 7.8 Unit-test that skirmish kills do not advance saved quest progress
- [ ] 7.9 Unit-test `SessionClock` play-time accumulation across save/load and auto-save gating

## 8. Documentation and verification

- [ ] 8.1 Replace `docs/mission.md` with `docs/quests.md` describing both quest kinds, modes, generation, notifications, and persistence
- [ ] 8.2 Update `docs/scenes.md`, `docs/ui.md`, `docs/save.md`, and `docs/config.md`
- [ ] 8.3 Remove the stale note in `docs/save.md` that `quests` is a reserved empty table
- [ ] 8.4 Run `openspec validate open-world-quests --strict` and targeted unit tests
- [ ] 8.5 Run `cargo dev` in the background to verify in-game behavior, then terminate it
