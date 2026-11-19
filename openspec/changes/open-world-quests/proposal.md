## Why

The game is transitioning from isolated arena missions (`core/src/mission/`, `MissionDef`, `BattleOutcome::Victory`) to an open-world experience. In an open world, destroying the last nearby enemy should not end the game, dying should not wipe world progress, and player progression should be driven by open-world quests persisted inside the active save slot.

## What Changes

- **BREAKING**: Remove `core/src/mission/` (`MissionDef`, `BattlePhase`, `BattleOutcome::Victory`, `BattleResult`, `BattleRestart`, victory/defeat end overlay, and `quick-battle.ron` mission flow for open-world saves).
- Replace player defeat game-over with **open-world respawn**: when the last player-team ship is destroyed, respawn the player at origin `(0, 0)` with a starter ship while preserving the world, explored state, and quest progress.
- Introduce a data-driven `core/src/quest/` feature plugin:
  - Hot-reloadable quest definitions loaded from `core/assets/config/quest/*.ron` (`QuestDef { id, title, description, requires, objectives }`).
  - v1 objective types: `DestroyEnemies { count }`, `ReachPosition { position, radius }`, `CollectItem { item, count }`.
  - Automatic prerequisite-based quest activation (`requires` all completed) and event-driven objective progression (`ShipDestroyed`, player position, cargo changes).
  - Persistence in the active save slot's SQLite `quests` table via `WorldStore`.
  - Compact HUD Quest Tracker displaying active quest title and live objective progress.
- Keep `QuickBattle` functioning as a lightweight unsaved skirmish arena (spawning player + test enemies in-memory without mission/campaign persistence).
- Update `/docs/mission.md` (replace with `/docs/quests.md`), `/docs/scenes.md`, `/docs/ui.md`, and `/docs/save.md`.

## Capabilities

### New Capabilities
- `open-world-quests`: Data-driven `.ron` quest definitions, prerequisite auto-activation, event-driven objective tracking (`DestroyEnemies`, `ReachPosition`, `CollectItem`), SQLite `quests` persistence, compact HUD tracker, and open-world player respawn at `(0, 0)`.

### Modified Capabilities
- `battle-outcome`: Remove arena `Victory` / `Defeat` end-screen transitions for open-world play; retain `ShipDestroyed` core-destruction events and replace last-player-ship destruction with origin respawn.

## Impact

- **Systems & Modules**:
  - Remove `core/src/mission/` (migrate `QuickBattle` arena spawn helper into a lightweight skirmish spawner).
  - New `core/src/quest/` plugin (`mod.rs`, `component.rs`, `def.rs`, `system.rs`, `hud.rs`).
  - `core/src/save/world/`: persist and restore `quests` table rows inside the active `WorldStore` transaction.
  - `core/src/scene/game/`: mount compact HUD Quest Tracker (`DespawnOnExit(SceneState::Game)`).
  - `core/assets/config/quest/`: starter open-world `.ron` quest definitions.
- **Dependencies**: Depends on `world-save-slots` (`WorldStore` SQLite `quests` table).
- **Documentation**: Replaces `docs/mission.md` with `docs/quests.md`; updates `docs/scenes.md`, `docs/ui.md`, `docs/save.md`.
