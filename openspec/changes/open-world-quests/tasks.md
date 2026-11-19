## 1. Quest Definitions & Validation

- [ ] 1.1 Create `core/assets/config/quest/quests.ron` with starter open-world quests covering `DestroyEnemies`, `ReachPosition`, and `CollectItem`
- [ ] 1.2 Create `core/src/quest/def.rs` implementing `QuestCatalogDef`, `QuestDef`, `QuestObjectiveDef`, `RonAssetLoader` registration, prerequisite/cycle validation, and `AssetEvent` bridge

## 2. Quest State Machine, Persistence & Open-World Respawn

- [ ] 2.1 Create `core/src/quest/component.rs` defining `QuestStatus`, `QuestProgress`, `QuestTracker`, `QuestCompleted`, `RespawnPoint`, and `QuestHudRoot`
- [ ] 2.2 Extend `WorldStore` (`SqliteWorldStore` and `InMemoryWorldStore`) in `core/src/save/world/` to load and save `quests` table rows atomically with world saves
- [ ] 2.3 Create `core/src/quest/system.rs` implementing prerequisite auto-activation, event-driven objective updates (`DestroyEnemies`, `ReachPosition`, `CollectItem`), and `respawn_player_on_defeat` at `(0, 0)`
- [ ] 2.4 Create `core/src/quest/hud.rs` and `core/src/quest/mod.rs` (`QuestPlugin`) rendering the compact HUD Quest Tracker (`DespawnOnExit(SceneState::Game)`) and registering `QuestPlugin` in `core/src/main.rs`

## 3. Remove Legacy Mission System

- [ ] 3.1 Migrate `QuickBattle` initial skirmish spawning into `core/src/scene/game/skirmish.rs` and delete `core/src/mission/` (`MissionPlugin`, `MissionDef`, `BattlePhase`, `BattleOutcome`, end overlay)

## 4. Tests, Documentation & Verification

- [ ] 4.1 Add unit tests for `QuestCatalogDef` validation (including cycle detection), prerequisite auto-activation, all 3 objective types, SQLite `quests` persistence round-trip, and open-world respawn on last-player-ship destruction
- [ ] 4.2 Replace `/docs/mission.md` with `/docs/quests.md` and update `/docs/scenes.md`, `/docs/ui.md`, and `/docs/save.md`
- [ ] 4.3 Run targeted unit tests (`cargo test`) and verify in-game with `cargo dev`
