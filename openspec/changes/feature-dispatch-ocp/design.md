## Context

Adding a block feature currently requires editing 10 separate `match feature` sites across `content/block/`, `config/block/`, `content/ship/ai/`, `scene/editor/`, and `scene/game/inspector/`. Three architectural issues compound this:
1. `content/block/component.rs:18` imports `Feature` → `TurretComponent`, while `feature/turret/component.rs:6-7`, `feature/turret/system.rs:4,9-10`, and `feature/shield/system.rs:5` import back from `content::block::component`, creating a real module cycle despite `openspec/specs/module-boundaries/spec.md`.
2. `Shield::insert` takes 4 unused parameters (`_visual`, `_pixel_size`, `_team`, `_config`) forced by `Feature::insert`'s ad-hoc parameter list, and `Storage(u32)` is a bare tuple variant instead of a typed struct.
3. `spawn_block_entity` (`block/component.rs:1188-1206`) spawns extra textures as children of `visual` and then calls `feature.insert(...)` for every feature, causing `Texture::insert` to overwrite `visual`'s primary `Sprite` with the last texture.

## Goals / Non-Goals

**Goals:**
- Make adding a block feature a single-module implementation of `BlockFeature` plus one line in `feature_dispatch!`.
- Keep static dispatch, `#[derive(Reflect, Clone, PartialEq)]`, and compiler-enforced exhaustiveness.
- Eliminate all `feature/* → block::component` reverse imports so `content::block` dependencies are strictly acyclic.
- Remove the multi-texture overwrite bug, dead `RecoilComponent`/`HeatEmitterComponent` types, and non-compliant `Component` suffixes (`TurretComponent`, `ComponentPlugin`).

**Non-Goals:**
- Runtime dynamic plugin registration (`FeatureTable`) or trait objects (`Box<dyn BlockFeature>`).
- Changing child entity spawning mechanics (handled separately in `unified-child-entity-observers`).

## Decisions

### 1. `BlockFeature` Trait + `feature_dispatch!` Declarative Macro
- **Choice**: Define a small trait `BlockFeature` in `content/block/feature/trait.rs` (or `component.rs`) with default no-op methods:
  ```rust
  pub struct SpawnCtx<'a> {
      pub base: Entity,
      pub visual: Entity,
      pub pixel_size: Vec2,
      pub team: Team,
      pub config: &'a GameConfigValue,
  }

  pub trait BlockFeature {
      fn install(&self, _commands: &mut Commands, _ctx: &SpawnCtx<'_>) {}
      fn accumulate_stats(&self, _stats: &mut BlockStats) {}
      fn validate(&self, _block_type: &str) -> Result<(), String> { Ok(()) }
      fn icon(&self) -> Option<&Handle<Image>> { None }
      fn texture(&self) -> Option<&Texture> { None }
  }
  ```
  Each feature struct (`Texture`, `TurretSpec`, `Generator`, `Battery`, `Consumer`, `Booster`, `Thruster`, `Storage`, `Shield`) implements `BlockFeature` inside its own submodule. A single declarative macro `feature_dispatch!` generates the `Feature` enum and forwards all `BlockFeature` methods without manual 9-arm matches.
- **Alternatives Considered**:
  - *Runtime `FeatureTable` / `FeatureHandler` registry*: Adds startup registration boilerplate and runtime lookup indirection without needing dynamic third-party mod loading yet.
  - *Trait objects (`Box<dyn BlockFeature>`)*: Breaks `Reflect`, `PartialEq`, and `Clone` on `BlockDef`, and requires `Any` downcasting in AI capability analysis.
  - *Keeping manual match arms*: Continues scattering stats, validation, and icon logic across multiple files.

### 2. Acyclic Leaf Types (`BlockStats`, `Health`, `Weapon` / `Pattern`)
- **Choice**:
  - Move `Health` into a leaf module `content/block/health.rs` (or `content/damage/`), `TurretMirror` into `feature/turret/`, and `ShootPattern`/`TargetingStrategy`/`ShootingProjectile` into `feature/turret/pattern.rs` (re-exported from `content/block/` so external callers remain clean).
  - Pass a lightweight leaf accumulator `BlockStats` into `BlockFeature::accumulate_stats(&self, stats: &mut BlockStats)` instead of importing `content::ship::ShipStats` into `content::block::feature`, preserving one-way `ship → block → feature` boundaries. `BlockRegistry::stats_from_grid` folds `BlockStats` into `ShipStats`.
  - Remove `#[derive(Component)]` from `Feature` (it is never inserted as an ECS component; only feature data components are inserted).

### 3. Multi-Texture Handling & Clean `Storage` Type
- **Choice**:
  - Replace `Feature::Storage(u32)` with `Storage { pub capacity: u32 }` in `feature/storage/component.rs` implementing `BlockFeature`.
  - In `spawn_block_entity`, handle primary vs. extra `Texture` layers through `feature.texture()` in one pass (first texture sets `visual`'s `Sprite` and block `z`; subsequent textures spawn ordered child `Sprite`s; `Texture::install` becomes a no-op or only installs the primary sprite when signaled via `SpawnCtx`), preventing any overwrite when multiple `Texture` features are configured.

## Risks / Trade-offs

- **Macro error diagnostics** → Keep `feature_dispatch!` simple (pure method-forwarding arms, no token-tree wizardry) so compiler errors point directly to the missing `BlockFeature` impl on the concrete type.
- **`BlockStats` vs `ShipStats` mapping** → `BlockStats` contains only the block-contributed fields (`mass`, `health`, `min_range`, `max_range`, `power_demand`, `power_production`, `power_capacity`, `cargo_capacity`, `thrust_power`, `rotate_speed`), keeping the mapping trivial and unit-tested.
- **Splitting large `content/block/component.rs` (1,408 lines)** → Moving `Health` and `ShootPattern` out also reduces `block/component.rs` toward the `AGENTS.md` 300-line guideline.
