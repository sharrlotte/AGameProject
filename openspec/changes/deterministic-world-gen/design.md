## Context

Audit of `core/src/quest/generator.rs:8-61`, `core/src/quest/system.rs:221-264`, `core/src/resource/mod.rs:18-39`, and `core/src/save/world/system.rs:64` confirmed 5 independent sources of non-determinism on the same world seed:
1. `generator::build` draws sequentially from shared `&mut GameRng` (3–9 draws per chunk) without hashing chunk coordinates.
2. `handle_save_world_event` writes `seed: rng.seed()`, overwriting `world_meta.seed` with the mutated xorshift64* cursor on every save.
3. Unpersisted `Offered` quests advance `GameRng` before reload, so revisiting the same chunk generates a different quest.
4. `max_active_generated` skips generation (and RNG draws) depending on when the player accepts/completes quests.
5. `GameRng::new(0)` is a zero fixed point in raw xorshift64*.

Meanwhile, generating a quest on every discovered chunk floods the player when traversing multiple chunks, and hardcoded `rng.next_u64() % 3` in `generator.rs:25` gives designers no control over biome, danger tier, or faction quest pools.

## Goals / Non-Goals

**Goals:**
- Guarantee that for any `(world_seed, WorldGenConfig, ChunkCoord)`, `sector_profile(coord)` and `quest_for_chunk(coord)` return the **exact same result** regardless of visit order, save/reload cycles, active quest counts, or combat RNG.
- Provide designer-tunable, hot-reloadable RON control over biomes, danger gradients, faction territories, quest templates, and quest spacing density.
- Eliminate quest spam via O(1) Jittered Macro-Grid spacing.
- Keep `world_meta.seed` strictly immutable across the entire lifetime of a save slot.

**Non-Goals:**
- Rendering biome backdrops or spawning faction-specific enemy fleets (those systems will consume `WorldGen::sector_profile` in later proposals).
- Replacing transient gameplay `GameRng` for combat/visual randomness (that remains separate from immutable `WorldSeed`).

## Decisions

### 1. Immutable `WorldSeed` + Hybrid `WorldGen` (`noise` crate + Keyed `SplitMix64` `ChunkRng`)
- **Choice**:
  - Introduce `#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)] pub struct WorldSeed(pub u64);` set once on `CreateSlot` and restored verbatim from `WorldMeta.seed` on load; `handle_save_world_event` ALWAYS writes `seed: world_seed.0` (never `rng.seed()`).
  - Add the `noise` crate (`OpenSimplex` / `SuperSimplex`, `Fbm`, `Worley`) wrapped inside `WorldGen` (`core/src/world/gen/`):
    - Derive distinct `u32` noise seeds per macro-field from `(world_seed, Domain::{Biome, Danger, Richness, Faction})` via `mix_seed`.
    - `danger(coord)`: radial distance factor from origin `(0, 0)` blended with low-frequency 2D `Fbm<SuperSimplex>` clamped to `[0.0, 1.0]` (configured in RON: `danger_distance_scale`, `danger_noise_weight`, `danger_frequency`).
    - `richness(coord)`: 2D `Fbm<SuperSimplex>` normalized to `[0.0, 1.0]`.
    - `biome(coord)` and `faction(coord)`: 2D `Worley` (cellular Voronoi) cell ID mapped deterministically into configured RON biome/faction tables (guaranteeing coherent contiguous regions and borders rather than salt-and-pepper noise).
  - Provide `ChunkRng` (SplitMix64 state initialized from `mix_seed(world_seed, domain as u64, coord.x(), coord.y())` with non-zero state fallback `0x9E37_79B9_7F4A_7C15`) for discrete per-chunk picks.
  - Also guard `GameRng::new(0)` with the same non-zero fallback so `seed == 0` never stalls.
- **Rationale**: Combines smooth, coherent macro-geography (`noise` crate) with uncorrelated, order-independent discrete rolls (`SplitMix64`). Each subsystem gets its own `Domain` salt so adding stations, asteroids, or POIs later never shifts existing quest generation.

### 2. Jittered Macro-Grid Quest Spacing (O(1), Non-Clustering)
- **Choice**: Configure `quest_grid_stride: i32` (default e.g. `3`, validated `>= 1`), optional `quest_grid_margin: i32` (validated `margin * 2 < stride`), and `quest_spawn_chance: f32` (`0.0..=1.0`) in RON:
  ```rust
  pub fn chunk_hosts_quest(&self, coord: ChunkCoord, cfg: &QuestSpacingConfig) -> bool {
      let stride = cfg.grid_stride.max(1);
      let cell = coord.0.div_euclid(IVec2::splat(stride));
      let mut rng = self.cell_rng(Domain::QuestPlacement, cell);
      if !rng.chance(cfg.spawn_chance) {
          return false;
      }
      let margin = cfg.grid_margin.clamp(0, (stride - 1) / 2);
      let span = (stride - 2 * margin).max(1);
      let offset = IVec2::new(
          margin + rng.range_i32(0, span),
          margin + rng.range_i32(0, span),
      );
      cell * stride + offset == coord.0
  }
  ```
- **Rationale**: Guarantees at most 1 quest per `stride × stride` chunk block (and with `margin >= 1` on `stride >= 3`, guarantees minimum 2-chunk separation so quests never cluster back-to-back!), runs in O(1) pure math with zero neighbor lookups, and is 100% invariant to visit order.
- **Alternative Considered**: *Pure per-chunk `rng.chance(p)`* — still clusters adjacent quests unpredictably.

### 3. Biome / Danger / Faction-Filtered RON Quest Templates
- **Choice**: Define validated RON templates in `QuestGenConfig`:
  ```rust
  pub struct QuestTemplateDef {
      pub id: String,
      pub title_template: String,
      pub description_template: String,
      pub biomes: Vec<BiomeId>,        // empty = matches any biome
      pub danger: (f32, f32),          // inclusive min..=max in [0.0, 1.0]
      pub factions: Vec<FactionId>,    // empty = matches any faction
      pub weight: f32,                 // > 0.0
      pub acceptance: QuestAcceptance,
      pub objectives: Vec<QuestObjectiveTemplate>,
  }
  ```
  - Given `coord`, `WorldGen::sector_profile(coord)` yields `{ biome, danger, richness, faction }`.
  - Filter `templates` matching `biome`, `danger` range, and `faction`. If none match, return `None` cleanly (zero fabricated quests).
  - Weighted-select one template using `world_gen.chunk_rng(Domain::QuestTemplate, coord)` and roll objective counts/radii/items (scaled optionally by `danger`/`richness`) using `world_gen.chunk_rng(Domain::QuestObjectives, coord)`.
  - `max_active_generated` is checked when **accepting** (`on_accept_offer`) or auto-activating (`Auto` template), NOT during pure world-seed placement — and `Offered` manual quests can still be shown with a clear notice/disabled Accept button if the player is at their active generated quest cap!

## Risks / Trade-offs

- **`noise` crate compile time & float portability** → `noise` is pure Rust with minimal transitive deps; note that `f64` math inside `noise` is deterministic across x86_64/ARM64 IEEE-754 platforms for standard operations; add golden-value unit tests locking sample `SectorProfile` and `QuestDef` outputs for fixed seeds.
- **Empty template match in extreme danger/biome combos** → Validation warns (or ensures a catch-all fallback template exists in shipped RON) while returning `None` gracefully rather than panicking.
- **Config consolidation synergy** → `WorldGenConfig` / `QuestGenConfig` cleanly nests inside `assets/config/game.ron` once `unify-config-and-settings` is applied.
