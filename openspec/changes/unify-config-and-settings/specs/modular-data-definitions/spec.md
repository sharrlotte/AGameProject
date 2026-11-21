## MODIFIED Requirements

### Requirement: Single-file encapsulation of data kinds
Every distinct configuration schema (`SimConfig`, `AiConfig`, `MinimapConfig`, `SkirmishConfig`, `QuestGenConfig`, `GameSettings`, and block feature configs) and save snapshot entity SHALL be defined in its own dedicated Rust module under 300 lines encapsulating its struct definitions, serde annotations, `Validate` implementation, and domain conversions. On disk, small read-only game tuning sections SHALL compose cleanly inside `assets/config/game.ron`, large catalogs (`block.ron`, `quests.ron`) SHALL remain standalone RON files, and user preferences SHALL live in `<data_dir>/settings.ron`.

#### Scenario: Block feature config encapsulation
- **WHEN** a developer inspects or modifies `TurretComponentConfig`
- **THEN** all turret-related schema, validation checks, and conversion code reside exclusively in `core/src/config/block/feature/turret.rs`

#### Scenario: Consolidated game.ron delegates to modular Rust schemas
- **WHEN** `core/assets/config/game.ron` is decoded into `GameConfigDef`
- **THEN** each nested section (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`) deserializes and validates through its own dedicated Rust module
