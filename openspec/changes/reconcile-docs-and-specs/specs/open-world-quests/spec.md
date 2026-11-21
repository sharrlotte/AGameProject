## MODIFIED Requirements

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

### Requirement: QuickBattle remains a functional unsaved skirmish

`QuickBattle` SHALL spawn a player fleet and test enemies in memory without campaign persistence. Quest activation and objective progression SHALL NOT run while `WorldContext` is `QuickBattle`. Player respawn SHALL still function in `QuickBattle`.

#### Scenario: Skirmish kills do not alter saved quest progress
- **WHEN** an enemy is destroyed during `QuickBattle`
- **THEN** no quest objective in the active save slot advances

#### Scenario: Skirmish defeat respawns the player
- **WHEN** the last player ship is destroyed during `QuickBattle`
- **THEN** a starter player ship is respawned and play continues
