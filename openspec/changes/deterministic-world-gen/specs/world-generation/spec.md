## ADDED Requirements

### Requirement: WorldGen derives continuous sector profiles and discrete chunk RNG purely from WorldSeed
The engine SHALL provide a `WorldGen` resource initialized from an immutable `WorldSeed(u64)` (matching `WorldMeta.seed`). Using the `noise` crate (2D `Fbm<SuperSimplex>` and 2D `Worley`), `WorldGen::sector_profile(coord: ChunkCoord)` SHALL deterministically compute `SectorProfile { biome, danger, richness, faction }` as a pure function of `(world_seed, coord)`. Discrete per-chunk random rolls SHALL use `WorldGen::chunk_rng(domain, coord)` backed by `SplitMix64` keyed by `(world_seed, domain, coord.x, coord.y)` with a non-zero state guard, completely independent of shared `GameRng`, chunk discovery order, or save/reload timing.

#### Scenario: Identical seed yields identical sector profile and quest regardless of visit order
- **WHEN** Session A discovers chunks `(1, 0)` then `(0, 1)` and Session B discovers `(0, 1)` then `(1, 0)` using the same `WorldSeed` and RON config
- **THEN** both sessions compute identical `SectorProfile` and identical generated `QuestDef` for `(1, 0)` and for `(0, 1)`

#### Scenario: Zero seed does not degenerate
- **WHEN** `WorldSeed(0)` or `GameRng::new(0)` is constructed
- **THEN** both substitute a non-zero mixing constant and produce well-distributed non-zero outputs

### Requirement: Quest placement uses a deterministic Jittered Macro-Grid to prevent clustering
Generated quest placement SHALL be gated by a configurable Jittered Macro-Grid (`quest_grid_stride >= 1`, `quest_grid_margin`, and `quest_spawn_chance` in `[0.0, 1.0]`). Within each `stride × stride` chunk macro-cell, at most one chunk coordinate SHALL be selected deterministically from `(world_seed, Domain::QuestPlacement, cell_coord)` to host a quest.

#### Scenario: Macro-grid caps quest density to at most one per stride x stride block
- **WHEN** `quest_grid_stride = 3` and every chunk in a `3 × 3` macro-cell is queried
- **THEN** at most 1 of the 9 chunks returns `true` for hosting a quest

#### Scenario: Margin prevents adjacent quests across macro-cell boundaries
- **WHEN** `quest_grid_stride = 3` and `quest_grid_margin = 1`
- **THEN** no two quest-hosting chunks across neighboring macro-cells are orthogonally or diagonally adjacent

### Requirement: Quest templates are RON-configured and filtered by biome, danger range, and faction
Generated quests SHALL be selected from validated, hot-reloadable RON `QuestTemplateDef` entries specifying `biomes`, `danger: (min, max)`, `factions`, `weight > 0.0`, `acceptance`, and objective templates. Selection SHALL filter templates matching the chunk's `SectorProfile` and pick by weighted `ChunkRng` roll; when no template matches, no quest SHALL be generated for that chunk.

#### Scenario: Biome, danger, and faction filters control eligible quests
- **WHEN** chunk `C` has `SectorProfile { biome: AsteroidBelt, danger: 0.6, faction: Corsairs, .. }` and only template `"pirate_bounty"` matches that biome, danger range, and faction
- **THEN** chunk `C` generates a quest instantiated from `"pirate_bounty"` and never from non-matching templates

#### Scenario: Reloading with an unpersisted Offered quest regenerates the exact same quest
- **WHEN** a manual `Offered` quest at chunk `C` is not persisted, the game saves and reloads, and chunk `C` is revisited
- **THEN** `WorldGen` produces the exact same `QuestDef` (title, objectives, counts, offer position) as before reload
