## Context

`core/src/mission/` currently drives gameplay through a single-arena `MissionDef` (`quick-battle.ron`), `BattlePhase::{Idle, Loading, Active, Ended}`, and `BattleOutcome::{Victory, Defeat}` triggered when either side's live ship count reaches zero. With `world-save-slots` providing persistent open-world SQLite saves, an arena Victory/Defeat loop no longer fits: destroying nearby enemies should not end the world, and player death should respawn the player rather than terminating the session.

## Goals / Non-Goals

**Goals:**
- Remove `core/src/mission/` (`MissionDef`, `BattlePhase`, `BattleOutcome`, `BattleResult`, `BattleRestart`, Victory/Defeat end overlay) for open-world play while keeping `QuickBattle` working as a lightweight unsaved skirmish arena.
- Implement open-world player respawn: when the last player-team ship is destroyed (`ShipDestroyed`), respawn a starter player ship at origin `(0, 0)` while preserving world state, entities, and quest progress.
- Build a data-driven `core/src/quest/` plugin loading hot-reloadable `.ron` quest definitions from `core/assets/config/quest/`, with prerequisite auto-activation (`requires: Vec<String>`), v1 objectives (`DestroyEnemies`, `ReachPosition`, `CollectItem`), SQLite `quests` table persistence via `WorldStore`, and a compact HUD Quest Tracker.
- Keep `ShipDestroyed` core-destruction events intact for quests, respawn, and future systems.

**Non-Goals:**
- Manual Quest Log UI to accept/pin/abandon quests (v1 uses prerequisite-driven auto-activation).
- NPC dialogue, stations, or procedural quest generation (v1 uses authored `.ron` quest definitions; the `QuestDef` / `QuestState` schema is designed so procedural quests can insert `QuestState` later).
- Removing `QuickBattle` skirmish spawning.

## Decisions

### 1. Data-Driven Quest Definitions (`core/assets/config/quest/*.ron`)
- **Choice**:
  ```rust
  #[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
  #[serde(deny_unknown_fields)]
  pub struct QuestCatalogDef {
      pub quests: Vec<QuestDef>,
  }

  #[derive(Deserialize, Clone, PartialEq, Debug)]
  #[serde(deny_unknown_fields)]
  pub struct QuestDef {
      pub id: String,
      pub title: String,
      pub description: String,
      #[serde(default)]
      pub requires: Vec<String>,
      pub objectives: Vec<QuestObjectiveDef>,
  }

  #[derive(Deserialize, Clone, PartialEq, Debug)]
  pub enum QuestObjectiveDef {
      DestroyEnemies { count: u32 },
      ReachPosition { position: Vec2, radius: f32 },
      CollectItem { item: String, count: u32 },
  }
  ```
  - Validated on load: unique non-empty `id`s, non-empty `objectives`, `count >= 1`, `radius > 0.0`, no unknown `requires` IDs, and acyclic prerequisite graph.
- **Rationale**: Single hot-reloadable catalog asset (`core/assets/config/quest/quests.ron`) mirrors `GameConfig`/`AiConfig`, validates cleanly on load, and avoids fragile directory-globbing in Bevy's `AssetServer`.

### 2. Quest State Machine & Event-Driven Progression
- **Choice**:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub enum QuestStatus {
      Active,
      Completed,
  }

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct QuestProgress {
      pub quest_id: String,
      pub status: QuestStatus,
      pub objective_counts: Vec<u32>,
  }
  ```
  - Stored in a `QuestTracker` Bevy `Resource` (`HashMap<String, QuestProgress>`) and persisted to the `quests` table in `WorldStore` inside the same save transaction as `world_meta` and `active_state`.
  - **Activation**: Whenever `QuestCatalog` loads or any quest completes, any `QuestDef` not yet in `QuestTracker` whose `requires` are all `QuestStatus::Completed` transitions to `QuestStatus::Active` with zeroed `objective_counts`.
  - **Objective updates** (in `FixedPostUpdate`, `SimulationSet::Death` / `SimulationSet::Stats`):
    - `DestroyEnemies`: increments on `ShipDestroyed` where `!event.team.is_player()`.
    - `ReachPosition`: checks distance from any live player-team ship core to `position <= radius` (marks `1 / 1`).
    - `CollectItem`: tracks total player-team cargo item count for `item` (or cumulative pickups) up to `count`.
  - **Completion**: When every objective reaches its target, `status` becomes `QuestStatus::Completed`, emitting a `QuestCompleted { quest_id }` message and immediately checking newly unlocked quests.

### 3. Open-World Player Respawn & Removing `core/src/mission/`
- **Choice**:
  - Replace `check_battle_outcome` with `respawn_player_on_defeat` (`FixedPostUpdate`, `SimulationSet::Death`, after `apply_body_mutations`, `in_state(SceneState::Game)`):
    - Consumes `ShipDestroyed`. If a player ship was destroyed and zero live player-team ships remain, spawns a fresh starter player ship from the active `ShipCatalog` at `(0.0, 0.0)` and re-attaches `ControlledShip` + `CameraTarget` to the respawned ship.
    - All world entities, chunks, and `QuestTracker` progress remain untouched.
  - Enemy destruction no longer ends the game (`BattleOutcome::Victory` and `BattlePhase::Ended` overlay are removed).
  - Move `QuickBattle` initial skirmish spawn into a small `core/src/scene/game/skirmish.rs` helper so `core/src/mission/` can be deleted cleanly.

### 4. Compact HUD Quest Tracker
- **Choice**: Render a compact top-left / below-FPS (or right-below-minimap) read-only HUD panel (`DespawnOnExit(SceneState::Game)`) listing active quests and their objective lines (e.g., `Destroy Enemies: 2/5`, `Reach Sector Beacon: 0/1`), updating only when `QuestTracker` is marked changed (`resource_changed::<QuestTracker>`).
- **Rationale**: Zero per-frame UI churn (`resource_changed`), clean layout that doesn't collide with the top-right minimap or left `ShipList`.

### 5. Module Layout (`AGENTS.md` Compliant)
```
core/src/quest/
├── mod.rs          # QuestPlugin, QuestCompleted message
├── component.rs    # QuestTracker, QuestProgress, QuestStatus, QuestHudRoot
├── def.rs          # QuestCatalogDef, QuestDef, QuestObjectiveDef + validation + AssetEvent bridge
├── system.rs       # activate_available_quests, update_quest_objectives, respawn_player_on_defeat + unit tests
└── hud.rs          # setup_quest_hud, update_quest_hud
```

## Risks / Trade-offs

- **Quest Catalog Changes Between Saves**: If a saved `QuestProgress` references a removed quest ID or a quest whose objective count changed, loading must handle it gracefully. → Unknown quest IDs are preserved in `QuestTracker` without crashing; `objective_counts` is resized/clamped to match `QuestDef.objectives.len()`.
- **Respawn Collision at `(0, 0)`**: An enemy might be sitting at `(0, 0)` when the player respawns. → Acceptable for v1; future station/safe-zone systems can supply a dynamic respawn point resource (`RespawnPoint(Vec2)` defaulting to `Vec2::ZERO`).
