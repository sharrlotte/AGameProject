## Why

`Feature::insert` (`core/src/content/block/feature/component.rs:28-49`) is a closed 9-arm match, and feature-specific behavior is scattered across 10 match sites (`stats_from_grid`, `validate_ship`, `spawn_block_entity`, AI capability, editor placed/preview/icons, and ship inspector). In addition, `content::block::component` and `content::block::feature::turret::component` form an import cycle violating `module-boundaries`, and `spawn_block_entity` overwrites primary sprites when a block declares multiple textures.

## What Changes

- Define a cohesive `BlockFeature` trait with default no-op methods (`install`, `accumulate_stats`, `validate`, `icon`, `extra_texture`) implemented in each feature's owning module (`sprite`, `turret`, `power`, `thruster`, `storage`, `shield`).
- Introduce a declarative `feature_dispatch!` macro in `core/src/content/block/feature/` that generates `Feature` enum dispatch forwarding from a single declaration table, preserving static dispatch, `Reflect`, `Clone`, and `PartialEq` with zero runtime lookup overhead.
- **BREAKING** Rename `TurretComponent` → `TurretSpec` and `ComponentPlugin` → `FeaturePlugin` to comply with `AGENTS.md` naming conventions, and remove unused dead types (`RecoilComponent`, `HeatEmitterComponent`) and unused `Shield::insert` parameters.
- Break the `content::block::component` ↔ `content::block::feature::turret` import cycle by moving shared leaf types (`ShootPattern`, `TargetingStrategy`, `Health`, `TurretMirror`) into leaf modules so `feature/*` never imports `block::component`.
- Fix the multi-texture overwrite bug in `spawn_block_entity` (`block/component.rs:1188-1206`) so primary and extra `Texture` layers never collide.
- Replace scattered `Feature::*` matches in `stats_from_grid`, `validate_ship`, editor palette icons, and ship inspector with `Feature` trait-dispatched methods.

## Capabilities

### New Capabilities
- `feature-dispatch`: Trait-and-macro static dispatch (`BlockFeature` + `feature_dispatch!`) consolidating block feature installation, stat accumulation, validation, and icon extraction behind one owning-module contract.

### Modified Capabilities
- `module-boundaries`: Eliminate the remaining `content::block::component` ↔ `content::block::feature::turret` / `shield` reverse imports and enforce leaf-only dependencies for `BlockFeature` implementations.

## Non-Goals

- Replacing the `Feature` data enum with heap-allocated trait objects (`Box<dyn BlockFeature>`).
- Changing how feature child entities (barrels, barriers, overdrive fields) are spawned (covered by `unified-child-entity-observers`).
- Refactoring AI role scoring internals beyond reading feature accessors cleanly.

## Impact

- `core/src/content/block/` (`component.rs`, `mod.rs`, leaf types for `Health`/`ShootPattern`/`TargetingStrategy`).
- `core/src/content/block/feature/` (`component.rs`, `mod.rs`, and submodules `sprite/`, `turret/`, `power/`, `thruster/`, `storage/`, `shield/`).
- `core/src/config/block/feature/` (`mod.rs`, `turret.rs`).
- `core/src/scene/editor/` (`ui.rs`, `grid.rs`, `ship_list.rs`) and `core/src/scene/game/inspector/system.rs`.
- `docs/architecture.md` (updated block feature architecture spec).
