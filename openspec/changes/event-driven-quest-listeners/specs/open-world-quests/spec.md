## MODIFIED Requirements

### Requirement: Objective progress is keyed by stable IDs and advances from gameplay events

`QuestProgress.objective_counts` SHALL be a map keyed by each objective's stable `id`. Active objectives SHALL update via dedicated event-gated listener systems: `DestroyEnemies { count }` in `on_ship_destroyed` (`.run_if(on_message::<ShipDestroyed>)`) for non-player teams; `CollectItem { item, count }` in `on_item_collected` (`.run_if(on_message::<ItemCollected>)`) cumulatively from the actual collected amount; and `ReachPosition { position, radius }` in a throttled `check_reach_position` system running only while at least one active `ReachPosition` objective exists. When every objective reaches its target, the quest SHALL transition to `Completed` and emit `QuestCompleted`, triggering reactive prerequisite activation.

#### Scenario: Destroying enemies completes a quest
- **WHEN** an active quest requires 2 enemy kills and two `ShipDestroyed` events for non-player teams are received
- **THEN** `on_ship_destroyed` advances that objective to `2/2` and emits `QuestCompleted`

#### Scenario: Flying within radius completes ReachPosition
- **WHEN** an active quest requires reaching `(1000, 0)` within radius `200` and a player ship reaches `(900, 0)`
- **THEN** `check_reach_position` advances that objective to `1/1` and emits `QuestCompleted`

#### Scenario: Collecting cargo advances CollectItem cumulatively
- **WHEN** an active quest requires 10 steel and the player collects 6 steel, spends it, then collects 4 more
- **THEN** `on_item_collected` advances the objective to `10/10` and emits `QuestCompleted`

#### Scenario: Reordering objectives preserves progress
- **WHEN** a quest's objectives are reordered in `quests.ron` and the catalog reloads
- **THEN** each objective's progress remains attached to its own stable id
