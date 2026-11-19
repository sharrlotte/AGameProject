## ADDED Requirements

### Requirement: Quests are loaded from validated hot-reloadable RON definitions
The quest system SHALL load `QuestCatalogDef` from `core/assets/config/quest/quests.ron` with hot-reload support. Validation SHALL reject duplicate or empty quest IDs, empty objective lists, non-positive objective counts or radii, unknown prerequisite IDs, and prerequisite cycles.

#### Scenario: Valid quest catalog loads and validates
- **WHEN** `quests.ron` defines quests with unique IDs, valid `DestroyEnemies`, `ReachPosition`, and `CollectItem` objectives, and acyclic `requires`
- **THEN** validation succeeds and `QuestCatalogValue` is populated

#### Scenario: Cyclic prerequisite graph is rejected
- **WHEN** Quest A requires Quest B and Quest B requires Quest A
- **THEN** validation returns an explicit error naming the cycle

### Requirement: Quests activate automatically from prerequisites and progress from world events
When entering `SceneState::Game` in a save slot or when any quest completes, every quest whose `requires` IDs are all `QuestStatus::Completed` SHALL activate automatically. Active objectives SHALL update from gameplay state and events:
- `DestroyEnemies { count }`: increments when `ShipDestroyed` fires for a non-player team.
- `ReachPosition { position, radius }`: completes when any live player-team ship core is within `radius` of `position`.
- `CollectItem { item, count }`: reflects player-team cargo holdings for `item` up to `count`.
When all objectives of an active quest reach their targets, the quest SHALL transition to `QuestStatus::Completed` and emit `QuestCompleted`.

#### Scenario: Destroying enemies completes a DestroyEnemies quest and unlocks its successor
- **WHEN** Quest 1 (`DestroyEnemies { count: 2 }`) is active and Quest 2 has `requires: ["quest_1"]`, and 2 enemy ships emit `ShipDestroyed`
- **THEN** Quest 1 transitions to `QuestStatus::Completed`, emits `QuestCompleted`, and Quest 2 automatically becomes `QuestStatus::Active`

#### Scenario: Flying within radius completes ReachPosition
- **WHEN** an active quest has `ReachPosition { position: (1000, 0), radius: 200 }` and a player ship reaches `(900, 0)`
- **THEN** the objective reaches `1/1` and completes the quest if no other objectives remain

#### Scenario: Collecting cargo completes CollectItem
- **WHEN** an active quest has `CollectItem { item: "steel", count: 10 }` and player ships hold at least `10` steel
- **THEN** the objective reaches `10/10`

### Requirement: Quest progress persists inside the active save slot's SQLite database
`QuestTracker` SHALL be saved into and loaded from the `quests` table of the active `WorldStore` inside the same save transaction as world state.

#### Scenario: Active and completed quests round-trip across save and load
- **WHEN** Quest 1 is `Completed` and Quest 2 is `Active` at `1/3` progress, and the save slot is saved and reloaded
- **THEN** Quest 1 remains `Completed` and Quest 2 resumes `Active` at `1/3` progress

### Requirement: Compact HUD Quest Tracker displays active objectives
While in `SceneState::Game`, a compact HUD panel (`DespawnOnExit(SceneState::Game)`) SHALL display currently active quests and their objective progress counters, updating reactively when `QuestTracker` changes.

#### Scenario: HUD updates when objective progress changes
- **WHEN** an active quest's `DestroyEnemies` progress advances from `0/3` to `1/3`
- **THEN** the HUD Quest Tracker updates to display `1/3`
