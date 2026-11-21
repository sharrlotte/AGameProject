## MODIFIED Requirements

### Requirement: The world seed has one authoritative field
`WorldMeta.seed` SHALL be the sole immutable persisted world seed (`WorldSeed`) created once on slot creation and SHALL NEVER be overwritten by mutable `GameRng` cursor state on save. `WorldMeta.rng_blob` SHALL NOT exist. Slot listing, world loading, and `WorldGen` initialization SHALL all read the same immutable `WorldMeta.seed` value.

#### Scenario: World seed stays identical across multiple saves and RNG draws
- **WHEN** a new slot is created with seed `S`, gameplay advances `GameRng` and discovers chunks, and the world is saved and reloaded multiple times
- **THEN** `world_meta.seed` remains strictly equal to `S` after every save and `WorldGen` initializes with `S` on every reload
