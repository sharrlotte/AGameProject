# AI Spec — AI Ships

## Components

`AIShipController { last_behavior, behavior, initial_behavior: AIShipBehavior,
timer: Repeating(1/15 s) }`. Inserted on every ship by `ShipData::spawn` with `Idle`.

`Timer` gates all transitions — AI evaluates 15× per second.

## `AIShipBehavior`

```
Idle | Wander | Attack { target_position: Vec2 } | Patrol { around_position: Vec2, radius: f32 }
Chase { target: Entity } | Flee { from: Entity } | Guard { target: Entity, radius: f32 }
Follow { target: Entity, radius: f32 } | Kamikaze { target: Entity } | Regroup | Kiting
```

**Behavior transitions (current implementation, evaluated in `FixedUpdate::control_ship`):**

| State | Transitions |
|---|---|
| `Regroup` | if allies collected (`teams` map filtered by team) within `(ally.size + own.size) * 2` → `Wander`; else inserts `ShipBehavior::Approach { position: ally_pos, radius: (ally.size + own.size) * 2 }`. If no allies → `Idle` |
| `Idle` | if an enemy is within `ship.max_range` → `Chase { target: enemy }`; else Approach world-origin with radius `size*2` |
| `Chase` | if target lost → `Idle`. Inserts `Approach { radius: min_range*0.9 }` if far, else `GoAround { radius: min_range*0.9 }` |
| `Wander` | RNG: 50% `Attack{0}`, else 50% `Guard` on last ally, else `Patrol{0, radius: size*10}` |
| `Attack` | inserts `Approach { position: target_position, radius: size }` |
| `Flee` | inserts `Approach` away from `from` by 1000 px |
| `Patrol` | `Approach { around_position, radius }` |
| `Guard` / `Follow` / `Kamikaze` | if target valid → `Approach`; else `previous()` (back to previous behavior) |
| `Kiting` | **`todo!()` — panics** if reached. Nothing transitions into `Kiting` in current code. |

`Idle`/`Chase` use `ship.min_range` (= `10000.0` seed lowered only by turret features)
and `ship.max_range` (= `1000.0` seed raised by turret feature ranges).

Skipped conditions: ships being followed (`CameraTarget.following`) or controlled
(`ControlledShip.controlling`) are skipped by AI.

Only teams 0 (player) and 1 are ever spawned (`for i in 0..2` in `check_ship`).
Schematics for AI ships come from `load_saved_schematics()` at random and are spawned
at a uniform random position in `[-10000, 10000]²` on `1000.0 px` grid steps.

## Movement application

`PostUpdate::update_ship_behavior` converts `ShipBehavior` into `ExternalForce`:

- `Approach`: computes angle error between `forward` and direction-to-target; applies
  torque (clamped `turn_speed`) with a squared torque multiplier on angular velocity;
  thrusts forward if `angle_diff < π/3` and `distance > radius`; applies deceleration
  model `stop_distance = v² / (2 * LINEAR_DAMPING)`; zeroes force when inside it.
- `GoAround`: steers to a ring point at `radius`, shifted 45° forward to orbit; similar
  torque logic; forces away from target when inside ring.

`ShipBehavior` is an `Approach`/`GoAround { position, radius }` component written by AI
state logic. It is never removed intentionally; overwritten on next transition.

Player-controlled ships skip `update_ship_behavior`.

## Camera assist

`PlayerControlShip` handles scroll-wheel zoom (`MIN_ZOOM = 0.2`, `MAX_ZOOM = 30.0`) and
WASD/arrow camera pan (with Shift boost ×3). When `CameraTarget.following` is set, the camera
smoothly lerps toward the followed ship's core position
(`current.lerp(target, 1 - exp(-10 * dt))`).

## Selection

`SelectablePlugin`: `Ctrl`+`Left-drag` selects player ships within the drag box;
`SelectableState::Selecting` tracks start pos; `Selection` takes entity under the box via
`BoundingBox` test + translation test (may double-count — see defects.md), and emits
`EntitySelectEvent { Vec<Entity> }`.

Consumers: Game HUD `update_select_panel` (dumps `"<TYPE>: <n>"` entries into the
selection panel).
