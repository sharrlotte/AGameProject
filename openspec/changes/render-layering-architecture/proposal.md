## Why

Currently, visual depth ($Z$) is inconsistently managed across the codebase: turret barrels are assigned `z: 150.0` in `block.ron` while shield barriers are spawned at `LAYER_BULLET` (`z = 100.0`), causing shield energy envelopes to render underneath rotating turret blocks. Furthermore, the booster overdrive field is positioned at `z = 50.0`, and layers lack clear architectural strata. A unified constant layering system is required to ensure shield envelopes properly encapsulate vessels and turrets, projectiles and beams layer correctly, and future rendering passes have dedicated depth strata.

## What Changes

- Introduce standardized visual depth constants in `core/src/constant.rs` (`LAYER_DEBRIS = 2.0`, `LAYER_HULL = 10.0`, `LAYER_TURRET_BASE = 20.0`, `LAYER_TURRET_BARREL = 50.0`, `LAYER_OVERDRIVE_FIELD = 70.0`, `LAYER_BULLET = 100.0`, `LAYER_BEAM = 110.0`, `LAYER_DROP = 120.0`, `LAYER_SHIELD = 160.0`, `LAYER_EXPLOSION = 200.0`, `LAYER_FLASH_OVERLAY = 220.0`, `LAYER_BLOCK_ICON = 300.0`).
- Update turret barrel visual Z-depth in `core/assets/config/base/block.ron` from `150.0` to `50.0` (`LAYER_TURRET_BARREL`).
- Update shield barrier visual spawn translation in `core/src/content/block/feature/shield/system.rs` from `LAYER_BULLET` (`100.0`) to `LAYER_SHIELD` (`160.0`).
- Update overdrive aura field visual translation from `50.0` to `70.0` (`LAYER_OVERDRIVE_FIELD`).
- Update specifications in `docs/combat.md` and `docs/simulation.md`.

## Capabilities

### New Capabilities
- `render-layering`: Standardized visual depth strata ($Z$) across game entities, projectiles, shields, auras, and overlays.

### Modified Capabilities
- `shield-defense`: Shield barrier rendering depth changes from projectile layer (`100.0`) to shield envelope layer (`160.0`), rendering above hull, turrets, and projectiles.
- `overdrive`: Booster field aura depth changes from `50.0` to `70.0` (`LAYER_OVERDRIVE_FIELD`), rendering above hull blocks and turret barrels, and below shield envelopes.

## Impact

- `core/src/constant.rs`: Defines new layer depth constants and recalibrates `LAYER_OVERDRIVE_FIELD`.
- `core/assets/config/base/block.ron`: Recalibrates turret barrel textures from `150.0` to `50.0`.
- `core/src/content/block/feature/shield/system.rs`: Uses `LAYER_SHIELD` for barrier translation.
- `docs/combat.md` and `docs/simulation.md`: Updates layer specifications.
