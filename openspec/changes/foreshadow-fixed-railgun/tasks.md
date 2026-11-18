## 1. 1×4 Foreshadow Asset & Config

- [ ] 1.1 Generate `core/assets/texture/block/foreshadow.png` at `64×256` RGBA (`32×128` upscaled 2× `NEAREST`) with transparent background in the dark space-steel railgun style
- [ ] 1.2 Update `foreshadow` in `core/assets/config/base/block.ron` to `shape: Rect((1, 4))`, `barrels: [(0.0, 128.0)]`, and `aim_tolerance: Some(0.035)`

## 2. Configurable `aim_tolerance` & Ship Target Leading

- [ ] 2.1 Add `#[serde(default)] pub aim_tolerance: Option<f32>` to `TurretComponentConfig` (`core/src/config/block.rs`) and `pub aim_tolerance: f32` to `TurretComponent` / `Turret` (`core/src/content/block/feature/turret/component.rs`), defaulting to `PI / 16.0`
- [ ] 2.2 Update `update_turret_target` (`core/src/content/block/feature/turret/system.rs`) to use `turret.aim_tolerance` and call `lead_point` for `Ship` targets using target `Velocity`
- [ ] 2.3 Add unit tests for custom `aim_tolerance` and ship target leading in `turret/system.rs`

## 3. Ship Rotation Aiming (`ShipBehavior::AimAt`)

- [ ] 3.1 Add `ShipBehavior::AimAt { target: Vec2, hold_distance: f32, aim_offset_angle: f32 }` to `core/src/content/ship/component.rs`
- [ ] 3.2 Implement `ShipBehavior::AimAt` in `update_ship_behavior` (`core/src/content/ship/system.rs`) to rotate the ship using `aim_offset_angle` and hold range
- [ ] 3.3 Update `apply_ai_behaviors` (`core/src/content/ship/ai/system.rs`) to detect fixed (`turn_speed == 0.0`) weapons on the ship, read their mounted `Rot90` angle, and emit `ShipBehavior::AimAt` instead of `GoAround` when in range
- [ ] 3.4 Add unit tests for `AimAt` alignment and `Rot90` offset calculation

## 4. Verification

- [ ] 4.1 Run `cargo test` for config parsing, turret targeting, and ship aiming tests
- [ ] 4.2 Run `cargo dev` to verify the 1×4 Foreshadow renders and AI ships rotate to aim and fire
