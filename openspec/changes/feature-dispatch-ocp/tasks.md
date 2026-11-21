## 1. Break Block ↔ Feature Import Cycle

- [ ] 1.1 Extract `Health` into a leaf module `core/src/content/block/health.rs` and re-export from `content::block`.
- [ ] 1.2 Move `ShootPattern`, `TargetingStrategy`, `ShootingProjectile`, pattern state components, and `TurretMirror` into `core/src/content/block/feature/turret/` (re-exporting where external callers expect them) so `feature/turret/*` and `feature/shield/*` have zero imports from `content::block::component`.
- [ ] 1.3 Delete unused `RecoilComponent` and `HeatEmitterComponent` from `core/src/content/block/component.rs`.

## 2. Implement `BlockFeature` Trait and `feature_dispatch!` Macro

- [ ] 2.1 Create `BlockStats`, `SpawnCtx`, and the `BlockFeature` trait (`install`, `accumulate_stats`, `validate`, `icon`, `texture`) in `core/src/content/block/feature/component.rs`.
- [ ] 2.2 Create `core/src/content/block/feature/storage/` (`mod.rs`, `component.rs`) with `pub struct Storage { pub capacity: u32 }` implementing `BlockFeature`.
- [ ] 2.3 Rename `TurretComponent` → `TurretSpec` and `ComponentPlugin` → `FeaturePlugin`, remove `#[derive(Component)]` from `Feature`, and implement `BlockFeature` in each feature submodule (`sprite`, `turret`, `power`, `thruster`, `storage`, `shield`).
- [ ] 2.4 Replace manual 9-arm matches on `Feature` with the `feature_dispatch!` declarative macro generating `Feature` and its forwarding methods.

## 3. Migrate Call Sites and Fix Multi-Texture Overwrite

- [ ] 3.1 Update `BlockRegistry::stats_from_grid` and `BlockRegistry::validate_ship` to call `feature.accumulate_stats(&mut block_stats)` and `feature.validate(&def.block_type)`.
- [ ] 3.2 Fix `spawn_block_entity` (`content/block/component.rs`) so multi-texture blocks assign the first texture to `Visual` and additional textures to child sprites without `Feature::install` overwriting `Visual`'s `Sprite`.
- [ ] 3.3 Update editor palette icon spawning (`scene/editor/ui.rs`) and ship inspector diagram spawning (`scene/game/inspector/system.rs`) to iterate `feature.icon()` instead of matching `Feature::Texture` / `Feature::Turret`.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests covering `BlockFeature` stat accumulation, validation, icon extraction, and multi-texture `spawn_block_entity` preservation.
- [ ] 4.2 Add or extend convention/boundary tests verifying zero `content::block::feature::*` imports of `content::block::component`.
- [ ] 4.3 Update `docs/architecture.md` (spec only, no notes) to reflect `BlockFeature`, `feature_dispatch!`, `FeaturePlugin`, and `TurretSpec`.
