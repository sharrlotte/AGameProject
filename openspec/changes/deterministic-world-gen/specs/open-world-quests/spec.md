## MODIFIED Requirements

### Requirement: Procedural generation is bounded and chunk-scoped

The generation system SHALL consume `SectorDiscovered` messages defined in `core/src/world/event.rs`. For each discovery, it SHALL derive `QuestId::Chunk(event.chunk)`, skip generation when that ID already exists in `QuestTracker`, check `WorldGen::chunk_hosts_quest` (Jittered Macro-Grid spacing), evaluate `WorldGen::sector_profile(event.chunk)`, and instantiate a matching RON `QuestTemplateDef` using `WorldGen`'s keyed `ChunkRng` (never shared `GameRng`). `max_active_generated` SHALL cap how many generated quests may be `QuestStatus::Active` simultaneously (blocking acceptance or auto-activation while at cap without mutating world seeds or suppressing deterministic placement).

#### Scenario: Discovery on a non-host chunk generates nothing
- **WHEN** `SectorDiscovered` is received for a chunk not selected by the Jittered Macro-Grid
- **THEN** no generated quest is created

#### Scenario: Discovery on a host chunk creates a deterministic template quest
- **WHEN** `SectorDiscovered` is received for a macro-grid host chunk with matching biome/danger/faction templates
- **THEN** one deterministic generated quest is added with `QuestStatus::Offered` (if `Manual`) or `QuestStatus::Active` (if `Auto` and below `max_active_generated`)

#### Scenario: Active generated quest cap limits activation without altering world determinism
- **WHEN** the number of active generated quests equals `max_active_generated` and the player discovers a manual quest host chunk
- **THEN** the manual quest remains deterministically generated as `Offered` (with acceptance gated until an active slot frees up), and discovering other chunks produces identical quests regardless of cap timing
