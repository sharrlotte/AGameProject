## ADDED Requirements

### Requirement: Each quest objective trigger runs in a dedicated event-gated listener
Quest progression SHALL NOT run as a single monolithic polling system. Each event-driven objective kind SHALL be handled by its own dedicated system gated by `.run_if(on_message::<T>)`:
- `on_ship_destroyed` gated by `on_message::<ShipDestroyed>` for `DestroyEnemies`
- `on_item_collected` gated by `on_message::<ItemCollected>` for `CollectItem`
- `on_sector_discovered` gated by `on_message::<SectorDiscovered>` for procedural quest discovery
- `check_reach_position` gated by a periodic timer and an active-`ReachPosition` run condition for `ReachPosition`

All objective listeners SHALL delegate progress mutation and completion checks to a shared helper that mutates `QuestTracker` only when progress changes and emits `QuestCompleted` in deterministic `QuestId::key()` order.

#### Scenario: Idle tick invokes zero event-gated quest listeners
- **WHEN** a fixed tick completes with no `ShipDestroyed`, `ItemCollected`, or `SectorDiscovered` messages and no active `ReachPosition` objective
- **THEN** none of the quest objective listeners execute and `QuestTracker` is not marked changed

#### Scenario: Kill and item pickup listeners operate independently
- **WHEN** an `ItemCollected` message is emitted without any `ShipDestroyed` message
- **THEN** only `on_item_collected` runs, advances matching `CollectItem` objectives, and emits `QuestCompleted` if all objectives are satisfied

### Requirement: Quest prerequisite activation is reactive instead of polling at 60 Hz
`activate_quests` SHALL NOT poll the quest catalog every fixed tick. It SHALL run reactively on session entry, catalog change, `QuestCompleted`, or `QuestActivated` (emitted when quests activate or when a manual offer is accepted), resolving transitive `QuestPrereqMode::Started` chains to a fixed point within one pass.

#### Scenario: Transitive Started tutorial chain activates in one pass
- **WHEN** Quest 1 unlocks Quest 2 (`Started`) and Quest 2 unlocks Quest 3 (`Started`), and a new save slot enters gameplay
- **THEN** `activate_quests` activates Quest 1, Quest 2, and Quest 3 in one reactive pass and emits `QuestActivated` for each

#### Scenario: Completing a quest unlocks its Completed-gated successor on the same tick
- **WHEN** `on_ship_destroyed` completes Quest A and emits `QuestCompleted`
- **THEN** `activate_quests` runs after the objective listeners in the same `FixedPostUpdate::Death` tick and activates Quest B requiring Quest A (`Completed`)
