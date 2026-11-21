# open-world-quests Specification

## Purpose
Defines requirements for authored and procedurally generated open-world quests, objective tracking, range-gated notifications, HUD display, and save-slot persistence.
## Requirements
### Requirement: Quests are loaded from validated hot-reloadable RON definitions

The quest system SHALL load `QuestCatalogDef` from `core/assets/config/quest/quests.ron` with hot-reload support. Validation SHALL reject duplicate or empty quest IDs, empty objective lists, non-positive objective counts or radii, unknown prerequisite IDs, prerequisite cycles, `QuestAcceptance::Manual` on an authored quest, and a `Manual` quest without an `offer`. Hot reload SHALL replace authored catalog entries only and SHALL preserve generated entries.

#### Scenario: Valid quest catalog loads and validates
- **WHEN** `quests.ron` defines authored quests with unique IDs, valid objectives, and acyclic `requires`
- **THEN** validation succeeds and `QuestCatalogValue` is populated

#### Scenario: Cyclic prerequisite graph is rejected
- **WHEN** Quest A requires Quest B and Quest B requires Quest A
- **THEN** validation returns an explicit error naming the cycle

#### Scenario: Manual authored quest is rejected
- **WHEN** an authored quest in `quests.ron` declares `acceptance: Manual`
- **THEN** validation rejects the catalog with an error naming that quest

#### Scenario: Hot reload preserves generated quests
- **WHEN** `quests.ron` changes while a generated quest exists in the catalog
- **THEN** authored entries are replaced and the generated entry remains unchanged

### Requirement: Quest identity and modes distinguish authored and generated quests

The quest system SHALL represent quest identity as `QuestId::{Authored(String), Chunk(IVec2)}`. Authored quests SHALL be auto-accepted; generated quest IDs SHALL derive from the chunk coordinate carried by `SectorDiscovered`. Each quest SHALL declare `QuestPrereqMode::{Completed, Started}` and `QuestAcceptance::{Auto, Manual}`. Authored tutorial quests SHALL use `Started` so a successor activates once its prerequisite is `Active` or `Completed`, allowing multiple tutorial quests to be active simultaneously without gating gameplay.

#### Scenario: Tutorial successor unlocks on start
- **WHEN** Quest 2 has `requires: ["quest_1"]` and `requires_mode: Started`, and Quest 1 becomes `Active`
- **THEN** Quest 2 becomes `Active` while Quest 1 is still `Active`

#### Scenario: Completion-gated successor waits
- **WHEN** a quest has `requires_mode: Completed` and its prerequisite is still `Active`
- **THEN** the quest remains locked until the prerequisite becomes `Completed`

#### Scenario: Generated quest id is chunk-derived
- **WHEN** `SectorDiscovered` is received for chunk `(3, -2)`
- **THEN** the generated quest uses `QuestId::Chunk(IVec2::new(3, -2))`

### Requirement: Objective progress is keyed by stable IDs and advances from gameplay events

`QuestProgress.objective_counts` SHALL be a map keyed by each objective's stable `id`. Active objectives SHALL update from gameplay state and events: `DestroyEnemies { count }` increments on `ShipDestroyed` for a non-player team; `ReachPosition { position, radius }` completes when any live player-team ship is within `radius` of `position`; `CollectItem { item, count }` increments from the `ItemCollected` message with the actual collected amount, making progress cumulative. When every objective reaches its target, the quest SHALL transition to `Completed` and emit `QuestCompleted`.

#### Scenario: Destroying enemies completes a quest
- **WHEN** an active quest requires 2 enemy kills and two `ShipDestroyed` events for non-player teams are received
- **THEN** that objective reaches `2/2` and the quest emits `QuestCompleted`

#### Scenario: Flying within radius completes ReachPosition
- **WHEN** an active quest requires reaching `(1000, 0)` within radius `200` and a player ship reaches `(900, 0)`
- **THEN** that objective reaches `1/1`

#### Scenario: Collecting cargo advances CollectItem cumulatively
- **WHEN** an active quest requires 10 steel and the player collects 6 steel, spends it, then collects 4 more
- **THEN** the objective reaches `10/10`

#### Scenario: Reordering objectives preserves progress
- **WHEN** a quest's objectives are reordered in `quests.ron` and the catalog reloads
- **THEN** each objective's progress remains attached to its own stable id

### Requirement: Procedural generation is bounded and chunk-scoped

The generation system SHALL consume `SectorDiscovered` messages defined in `core/src/world/event.rs`. For each discovery it SHALL derive a chunk-based `QuestId`, skip generation when that ID already exists in the catalog or tracker, and skip generation when the number of active generated quests has reached `max_active_generated` from `generation.ron`. Generated quests SHALL enter `QuestStatus::Active` when `acceptance` is `Auto` and `QuestStatus::Offered` when `acceptance` is `Manual`. With no `SectorDiscovered` emitter present, the system SHALL generate nothing.

#### Scenario: Discovery creates a manual offered or auto active quest
- **WHEN** `SectorDiscovered` is received for a chunk with no existing quest
- **THEN** one generated quest is created with `QuestStatus::Offered` if `Manual` or `QuestStatus::Active` if `Auto`

#### Scenario: Duplicate discovery is ignored
- **WHEN** `SectorDiscovered` is received again for a chunk that already has a generated quest
- **THEN** no additional quest is created

#### Scenario: Active generated quest cap is enforced
- **WHEN** the number of active generated quests equals `max_active_generated` and a new chunk is discovered
- **THEN** no additional generated quest is created

#### Scenario: Generation is dormant without an emitter
- **WHEN** no `SectorDiscovered` message is emitted during a session
- **THEN** no procedurally generated quest is created

### Requirement: Manual quests use range-gated notifications and a non-blocking accept dialog

A generated quest with `QuestAcceptance::Manual` SHALL carry a `QuestOffer { position, radius }`. While a live player-team ship is within `radius` of `position`, a notification card SHALL appear in a right-center queue; when no player ship is within range, the card SHALL be hidden while the quest remains `Offered`. Clicking the card SHALL reveal a dialog that does not pause gameplay and offers Accept and Close. Accept SHALL move the quest to `Active`; Close SHALL leave it `Offered` so it is offered again on a later visit. Auto-accepted quests SHALL NOT appear in the notification queue.

#### Scenario: Notification appears and hides with range
- **WHEN** a player ship enters and later leaves a manual quest's offer radius
- **THEN** the card appears on entry and is hidden on exit while the quest stays `Offered`

#### Scenario: Accept activates a manual quest
- **WHEN** the player clicks Accept in a manual quest dialog
- **THEN** the quest transitions from `Offered` to `Active`

#### Scenario: Close preserves the offer
- **WHEN** the player clicks Close in a manual quest dialog
- **THEN** the quest remains `Offered` and may be offered again on a later visit

#### Scenario: Dialog does not pause gameplay
- **WHEN** a notification card is clicked and its dialog is open
- **THEN** the game continues simulating without a `Modal::Pause` transition

### Requirement: Quest progress persists inside the active save slot's SQLite database

`QuestTracker` SHALL be saved into and loaded from the `quests` table of the active `WorldStore` in the same transaction as world state. `QuestRecord` SHALL contain the full quest definition and progress. Quests with `QuestStatus::Offered` SHALL NOT be saved. On load, an authored quest SHALL use the current RON definition when available (falling back to the saved definition if the ID was removed), and a generated quest SHALL use its saved definition.

#### Scenario: Active and completed quests round-trip
- **WHEN** one quest is `Completed` and another is `Active` at `1/3` progress, then the slot is saved and reloaded
- **THEN** the statuses and objective progress are restored

#### Scenario: Offered quests are not persisted
- **WHEN** a slot is saved while a manual quest is `Offered`
- **THEN** no row is written for that quest

#### Scenario: Authored definition authority
- **WHEN** an authored quest's RON definition changes and an old save is loaded
- **THEN** the quest uses the new RON definition

#### Scenario: Generated definition authority
- **WHEN** a generated quest is loaded from a save
- **THEN** its saved definition is used verbatim

### Requirement: Compact HUD Quest Tracker displays active objectives

While in `SceneState::Game`, a compact HUD panel SHALL display active quests and their objective progress counters, updating reactively when quest progress changes. The manual-offer notification queue SHALL be a separate right-center panel.

#### Scenario: HUD updates when objective progress changes
- **WHEN** an active quest's `DestroyEnemies` progress advances from `0/3` to `1/3`
- **THEN** the HUD Quest Tracker displays `1/3`

### Requirement: QuickBattle remains a functional unsaved skirmish

`QuickBattle` SHALL spawn a player fleet and test enemies in memory without campaign persistence. Quest activation and objective progression SHALL NOT run while `WorldContext` is `QuickBattle`. Player respawn SHALL still function in `QuickBattle`.

#### Scenario: Skirmish kills do not alter saved quest progress
- **WHEN** an enemy is destroyed during `QuickBattle`
- **THEN** no quest objective in the active save slot advances

#### Scenario: Skirmish defeat respawns the player
- **WHEN** the last player ship is destroyed during `QuickBattle`
- **THEN** a starter player ship is respawned and play continues

### Requirement: Session timekeeping survives mission removal

Play time SHALL be tracked by a `SessionClock` resource instead of `BattleResult`. Saving SHALL write the clock's elapsed time to `WorldMeta.play_time_secs`; loading SHALL resume the clock from the saved value. Auto-save SHALL run only while `SceneState::Game` and no pause modal is open (`ModalStack::top()` is not `Modal::Pause`).

#### Scenario: Play time accumulates across save and load
- **WHEN** a slot with 120 seconds of play time is saved, reloaded, and played for 10 more seconds
- **THEN** the next save records approximately 130 seconds

#### Scenario: Auto-save is gated by scene and pause state
- **WHEN** the game is in `SceneState::Game` but `ModalStack::top()` is `Some(Modal::Pause)`
- **THEN** the auto-save timer does not fire

