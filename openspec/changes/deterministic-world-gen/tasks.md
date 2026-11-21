## 1. Immutable `WorldSeed` and `noise` Dependency

- [ ] 1.1 Add `noise` dependency to `core/Cargo.toml`.
- [ ] 1.2 Add `WorldSeed(pub u64)` resource in `core/src/resource/mod.rs` and guard `GameRng::new(0)` against the xorshift64* zero fixed point.
- [ ] 1.3 Update `core/src/save/world/system.rs` (`handle_create_slot`, `handle_save_world_event`, `spawn_loaded_world`) so `WorldSeed` is set on create/load and `WorldMeta.seed` is always written from immutable `WorldSeed.0` (never mutated `rng.seed()`).

## 2. Build Deterministic `WorldGen` Module

- [ ] 2.1 Create `core/src/world/gen/` (`mod.rs`, `component.rs`, `lib.rs`) defining `Domain`, `BiomeId`, `FactionId`, `SectorProfile`, `WorldGenConfig` (noise frequencies, danger scaling, biome/faction tables, `QuestSpacingConfig`), `ChunkRng` (`SplitMix64` with `mix_seed` + non-zero guard), and `WorldGen`.
- [ ] 2.2 Implement `WorldGen::sector_profile(coord)` using `noise` crate `Fbm<SuperSimplex>` (danger, richness) and `Worley` (contiguous biome & faction territories).
- [ ] 2.3 Implement `WorldGen::chunk_hosts_quest(coord, spacing)` using the O(1) Jittered Macro-Grid (`grid_stride`, `grid_margin`, `spawn_chance`).
- [ ] 2.4 Implement `Validate` for `WorldGenConfig` and `QuestSpacingConfig` (`grid_stride >= 1`, `grid_margin * 2 < grid_stride`, `0.0..=1.0` chance/danger ranges, non-empty biome/faction tables).

## 3. Biome/Danger/Faction-Filtered RON Quest Templates

- [ ] 3.1 Extend `QuestGenConfig` (`core/src/quest/component.rs`, `def.rs`) with validated `QuestTemplateDef` entries (`biomes`, `danger`, `factions`, `weight`, `acceptance`, objective templates) and spacing config in RON (`generation.ron` / merged `game.ron`).
- [ ] 3.2 Refactor `quest::generator::build` (`core/src/quest/generator.rs`) to accept `&WorldGen`, check `chunk_hosts_quest`, filter templates against `sector_profile`, and roll template/objectives via `world_gen.chunk_rng(...)` (removing `&mut GameRng`).
- [ ] 3.3 Update `generate_from_discovery` (`core/src/quest/system.rs`) and `on_accept_offer` (`core/src/quest/hud.rs`) so `max_active_generated` caps active generated quests cleanly without altering deterministic chunk placement.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests proving: visit-order independence on the same seed, reload repeatability for unpersisted `Offered` quests, `WorldSeed` immutability across repeated saves, `seed == 0` non-degeneracy, Jittered Macro-Grid spacing/non-clustering guarantees, and biome/danger/faction template filtering.
- [ ] 4.2 Update `docs/quests.md`, `docs/save.md`, and `docs/architecture.md` (spec only, no notes) documenting `WorldSeed`, `WorldGen`, `SectorProfile`, Jittered Macro-Grid spacing, and RON quest templates.
