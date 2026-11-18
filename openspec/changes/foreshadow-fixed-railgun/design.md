## Context

Foreshadow is a railgun turret in `core/assets/config/base/block.ron` with `turn_speed: 0.0`, meaning `turret_tracking` (`core/src/content/block/feature/turret/system.rs`) never rotates its barrel. However:
1. `ShipBehavior` (`core/src/content/ship/component.rs`) only has `Approach` and `GoAround`. When an AI ship enters weapon range, `apply_ai_behaviors` (`core/src/content/ship/ai/system.rs`) switches to `GoAround`, which points the ship's nose perpendicular to the enemy.
2. `update_turret_target` checks `angle_error.abs() < PI / 16.0` (±11.25°) and only calls `lead_point` for `EnemyBullet`, not `Ship` targets.
3. Foreshadow is currently `Rect((3, 9))` with `barrels: [(0.0, 0.0)]` and a `128×128` sprite.

## Goals / Non-Goals

**Goals:**
- Resize Foreshadow to a `1×4` spinal railgun (`shape: Rect((1, 4))`) with a `64×256` transparent-background sprite (`32×128` upscaled 2× `NEAREST`) and muzzle barrel offset `(0.0, 128.0)`.
- Enable AI ships with fixed (`turn_speed == 0.0`) weapons to rotate the entire ship to aim at their target while holding range (`ShipBehavior::AimAt`).
- Respect the mounted `Rot90` orientation of the fixed gun when computing the ship's desired rotation.
- Support configurable `aim_tolerance` in `block.ron` and lead moving `Ship` targets in `update_turret_target`.

**Non-Goals:**
- Changing player-piloted (`ControlMode::Pilot`) manual steering controls.
- Adding new turret power or ammo mechanics to Foreshadow beyond its current config.
- Changing how rotating turrets (`turn_speed > 0.0`) behave.

## Decisions

### 1. `ShipBehavior::AimAt` for Fixed-Weapon Ship Aiming
Add `AimAt { target: Vec2, hold_distance: f32, aim_offset_angle: f32 }` (or `AimAt { target: Vec2, hold_distance: f32 }` with ship-level fixed-gun offset) to `ShipBehavior`.
- **Choice**: Store `aim_offset_angle: f32` (radians, derived from the primary fixed gun's `Rot90` placement on the ship) in `ShipBehavior::AimAt { target: Vec2, hold_distance: f32, aim_offset_angle: f32 }`.
- **Behavior in `update_ship_behavior`**:
  - Compute desired ship angle so that `ship_rotation + aim_offset_angle` points at `target`.
  - Apply the same PD torque control as `Approach` to smoothly rotate and settle on target.
  - Hold range around `hold_distance` using a deadband (thrust forward if `distance > hold_distance * 1.1` and roughly facing target; otherwise coast/pivot in place).
- **Selection in `apply_ai_behaviors`**:
  - When `AiBehavior::Attack` or `AiBehavior::Chase` has a target and the ship has at least one fixed weapon (`turn_speed == 0.0`) within `min_range`, emit `ShipBehavior::AimAt` aimed at the target's lead/core position instead of `GoAround`.

### 2. Configurable `aim_tolerance` & Ship Target Leading
- Add `#[serde(default)] pub aim_tolerance: Option<f32>` to `TurretComponentConfig` and `pub aim_tolerance: f32` (default `PI / 16.0`) to `Turret`.
- In `block.ron`, set `aim_tolerance: Some(0.035)` (~2°) on `foreshadow` so it only fires when accurately aligned.
- In `update_turret_target`, query `&Velocity` on target ships (with `Option<&Velocity>` or `&Velocity`) and pass it to `lead_point` when computing `candidate.aim` for `Ship` targets, matching `EnemyBullet`.

### 3. `1×4` Foreshadow Geometry & Sprite
- In `core/assets/config/base/block.ron`:
  - `shape: Rect((1, 4))`
  - `barrels: [(0.0, 128.0)]` (assuming `BLOCK_SIZE = 64.0`, a 1×4 block is `64×256` world units, so `+128.0` is the front muzzle edge).
  - `aim_tolerance: Some(0.035)`
- Generate `core/assets/texture/block/foreshadow.png` at `32×128` and upscale 2× with `NEAREST` to `64×256` RGBA, following `AGENTS.md` art style (transparent background, dark space-steel palette, no black outer outline, twin-rail electromagnetic barrel with cyan coils and gold core).

## Risks / Trade-offs

- **Multiple fixed guns at different `Rot90` angles**: If a ship mounts two fixed guns at different `Rot90` orientations, `apply_ai_behaviors` picks the first fixed gun's orientation. Acceptable for now; forward-mounted guns (`R0`) all share `0.0` offset.
- **Spinning / overshooting**: A heavy ship with low `rotate_speed` may oscillate around the target if `aim_tolerance` is too tight. Using the existing angular damping + PD torque settling (`angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01`) mitigates overshoot.
