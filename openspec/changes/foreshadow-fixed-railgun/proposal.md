## Why

The Foreshadow railgun is currently a 3×9 turret with `turn_speed: 0.0`, which prevents its barrel from rotating, but AI ships still use `GoAround` (orbiting broadside) and never rotate the ship to align the fixed gun with targets. Additionally, `update_turret_target` does not lead moving enemy ships and uses a wide `PI / 16.0` firing cone, causing long-range railgun shots to miss. Making Foreshadow a compact **1×4 fixed spinal railgun** that relies on ship rotation to aim creates a distinct sniper/spinal-weapon ship archetype.

## What Changes

- Resize `foreshadow` in `core/assets/config/base/block.ron` to `shape: Rect((1, 4))` with `barrels: [(0.0, 128.0)]` and a tight configurable `aim_tolerance`.
- Regenerate `core/assets/texture/block/foreshadow.png` as a `64×256` (drawn `32×128`, upscaled 2× `NEAREST`) transparent-background 1×4 railgun sprite matching the dark space-steel art style.
- Add `aim_tolerance: Option<f32>` to `TurretComponentConfig` / `Turret` (defaulting to `PI / 16.0`) and use `lead_point` for `Ship` targets in `update_turret_target`.
- Add `ShipBehavior::AimAt { target: Vec2, hold_distance: f32 }` and update `apply_ai_behaviors` + `update_ship_behavior` so AI ships with fixed (`turn_speed == 0.0`) weapons rotate to align the mounted gun's `Rot90` facing direction with the target while holding range.

## Capabilities

### New Capabilities
- `fixed-spinal-aim`: Ship-rotation aiming (`ShipBehavior::AimAt`) that aligns a ship's fixed (`turn_speed == 0.0`) weapon's mounted `Rot90` direction with its target, plus configurable `aim_tolerance` and ship-target leading in `update_turret_target`.
- `foreshadow-railgun-1x4`: 1×4 Foreshadow block configuration (`Rect((1, 4))`, muzzle barrel offset, tight `aim_tolerance`) and matching `64×256` pixel-art sprite.

### Modified Capabilities
- None.

## Impact

- Config & Assets: `core/assets/config/base/block.ron`, `core/assets/texture/block/foreshadow.png`
- Turret system: `core/src/config/block.rs`, `core/src/content/block/feature/turret/component.rs`, `core/src/content/block/feature/turret/system.rs`
- Ship movement & AI: `core/src/content/ship/component.rs`, `core/src/content/ship/system.rs`, `core/src/content/ship/ai/system.rs`
