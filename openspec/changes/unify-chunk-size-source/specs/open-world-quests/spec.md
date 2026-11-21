## MODIFIED Requirements

### Requirement: Quest identity and modes distinguish authored and generated quests

The quest system SHALL represent quest identity as `QuestId::{Authored(String), Chunk(ChunkCoord)}`. Authored quests SHALL be auto-accepted; generated quest IDs SHALL derive from the `ChunkCoord` carried by `SectorDiscovered`, and generated world positions SHALL use `ChunkCoord::center(CHUNK_SIZE)` from `core/src/constant.rs` rather than a duplicate `chunk_size` field in `QuestGenConfig`. Each quest SHALL declare `QuestPrereqMode::{Completed, Started}` and `QuestAcceptance::{Auto, Manual}`. Authored tutorial quests SHALL use `Started` so a successor activates once its prerequisite is `Active` or `Completed`, allowing multiple tutorial quests to be active simultaneously without gating gameplay.

#### Scenario: Tutorial successor unlocks on start
- **WHEN** Quest 2 has `requires: ["quest_1"]` and `requires_mode: Started`, and Quest 1 becomes `Active`
- **THEN** Quest 2 becomes `Active` while Quest 1 is still `Active`

#### Scenario: Completion-gated successor waits
- **WHEN** a quest has `requires_mode: Completed` and its prerequisite is still `Active`
- **THEN** the quest remains locked until the prerequisite becomes `Completed`

#### Scenario: Generated quest id and center derive from ChunkCoord and CHUNK_SIZE
- **WHEN** `SectorDiscovered` is received for `ChunkCoord::new(3, -2)`
- **THEN** the generated quest uses `QuestId::Chunk(ChunkCoord::new(3, -2))` and its offer position equals `ChunkCoord::new(3, -2).center(CHUNK_SIZE)`
