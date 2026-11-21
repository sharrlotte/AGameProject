## Why

Procedural quest generation is currently non-deterministic even on the exact same `world_meta.seed`:
1. `quest::generator::build` (`core/src/quest/generator.rs:8-41`) pulls 3–9 sequential values from the shared mutable `GameRng` without hashing chunk coordinates, so visiting `(1, 0)` then `(0, 1)` produces different quests than `(0, 1)` then `(1, 0)`.
2. `handle_save_world_event` (`core/src/save/world/system.rs:64`) overwrites `world_meta.seed` with the mutated `rng.seed()` cursor, destroying the original world seed on the first save.
3. Unpersisted `Offered` quests consume RNG draws before save/reload, `max_active_generated` skips draws based on acceptance timing, `GameRng` has a `seed == 0` fixed point, and every single discovered chunk tries to spawn a quest (flooding players who move across several chunks in one minute).
4. Designers have no RON-driven way to control which quests appear in which biome, danger level, or faction territory.

## What Changes

- Add the `noise` crate to `core/Cargo.toml` and build a deterministic `WorldGen` service/resource seeded strictly from the immutable `WorldSeed(u64)` (`world_meta.seed`, never overwritten on save!).
- Compute continuous spatial macro-fields (`SectorProfile { biome, danger: 0.0..=1.0, richness: 0.0..=1.0, faction }`) using seeded `noise` builders (2D fBm Simplex for danger/richness + Worley/cellular Voronoi for contiguous biomes and faction territories).
- Provide a pure, order-independent `ChunkRng` (`SplitMix64` keyed by `(world_seed, Domain, chunk.x, chunk.y)` with non-zero state guard) for discrete per-chunk rolls, and fix `GameRng::new(0)`'s zero fixed point.
- Control quest density via a deterministic **Jittered Macro-Grid** (`quest_grid_stride × quest_grid_stride` chunks + `spawn_chance`, configurable in RON) so at most one host chunk per macro-cell can spawn a quest, preventing back-to-back quest spam in O(1) time without neighbor queries.
- Replace hardcoded objective selection in `quest::generator::build` with RON-configurable `QuestTemplateDef` entries filtered and weighted by `(biomes, danger: (min, max), factions, weight, acceptance, objectives)`.
- Keep `max_active_generated` strictly as an active-quest acceptance cap (never suppressing or altering deterministic chunk quest seeds).

## Capabilities

### New Capabilities
- `world-generation`: Deterministic `WorldGen` engine combining `noise`-crate continuous spatial fields (`SectorProfile` with biome, danger, richness, faction), keyed `SplitMix64` per-chunk `ChunkRng`, Jittered Macro-Grid quest density spacing, and RON-filtered quest templates.

### Modified Capabilities
- `open-world-quests`: Replace sequential `GameRng` quest generation and per-chunk quest spawning with deterministic `(world_seed, ChunkCoord)` generation gated by Jittered Macro-Grid spacing and biome/danger/faction-filtered RON templates, with `max_active_generated` enforced on acceptance/activation rather than distorting world seeds.
- `save-integrity`: Guarantee `WorldMeta.seed` is immutable across saves (never overwritten by mutated `GameRng` state) and serves as the sole authoritative seed for `WorldGen`.

## Non-Goals

- Spawning physical biome terrain/nebula visuals or faction AI fleets (future features will query the same `WorldGen::sector_profile(coord)` API).
- Changing chunk storage tables or `ChunkCache` (handled by `chunk-storage-cache-and-layout`).

## Impact

- `core/Cargo.toml` (add `noise` crate).
- `core/src/world/gen/` (new feature module: `mod.rs`, `component.rs`, `lib.rs` with `WorldGen`, `SectorProfile`, `Biome`, `Faction`, `Domain`, `ChunkRng`, `mix_seed`, Jittered Macro-Grid helper).
- `core/src/quest/` (`component.rs`, `def.rs`, `generator.rs`, `system.rs`).
- `core/src/resource/mod.rs` (`WorldSeed`, non-zero `GameRng` guard).
- `core/src/save/world/system.rs` (preserve immutable `WorldMeta.seed` on save, initialize `WorldGen` on create/load).
- `core/assets/config/quest/generation.ron` (or `worldgen` section in `game.ron` once `unify-config-and-settings` lands).
- `docs/quests.md`, `docs/save.md`, `docs/architecture.md` (spec-only updates).
