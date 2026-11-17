# AI Spec — AI Ships

## Components

Each AI behavior is its own marker component in `content/ship/ai/component.rs`:

```
BehaviorIdle | BehaviorWander | BehaviorKiting | BehaviorRegroup
BehaviorChase { target: Entity } | BehaviorAttack { target_position: Vec2 }
BehaviorPatrol { around_position: Vec2, radius: f32 }
BehaviorFlee { from: Entity } | BehaviorGuard { target: Entity, radius: f32 }
BehaviorFollow { target: Entity, radius: f32 } | BehaviorKamikaze { target: Entity }
```

`ShipData::spawn` inserts `BehaviorTimer::new(config.ai_think_rate_hz)` and
`BehaviorIdle` on every ship. Each ship carries exactly one behavior marker;
transition systems remove all markers and insert the new one in one deferred
command, so `Query<&BehaviorChase>` (or `With<BehaviorChase>`) is the single
source of truth for the current behavior.

`BehaviorTimer` gates all transitions — AI evaluates 15× per second. Each
behavior system ticks the timer (only one system matches a given ship per
tick) and reads `ShipIndex`, a shared resource of per-team ship snapshots
rebuilt once per fixed tick by `build_ship_index`.

`AIShipController` and the `AIShipBehavior` enum no longer exist.

**Behavior transitions** (one system per behavior in
`content/ship/ai/system.rs`, registered in `AiPlugin`, all in
`SimulationSet::Simulate` and chained after `build_ship_index`):

| Component / system | Transitions |
|---|---|
| `BehaviorRegroup` | ally within `(ally.size + own.size) * ai_group_range_multiplier` → `Wander`; else `Approach { ally_pos, radius }`. No allies → `Idle` |
| `BehaviorIdle` | enemy within `ship.max_range` → `Chase { target }`; else `Approach { Vec2::ZERO, radius: size * ai_group_range_multiplier }` |
| `BehaviorChase` | target lost or self → `Idle`. Inserts `Approach { radius: min_range * ai_min_range_factor }` if far, else `GoAround` |
| `BehaviorWander` | RNG: `ai_wander_attack_chance` + enemy in range → `Kiting`; then `ai_wander_guard_chance` → `Attack { 0 }`; then same chance → `Guard` on last ally; else `Patrol { 0, radius: size * ai_wander_patrol_radius_multiplier }` |
| `BehaviorKiting` | nearest enemy within `max_range`: `Approach` away if closer than `max_range * ai_min_range_factor`, else `GoAround` at that radius; no enemy → `Idle` |
| `BehaviorAttack` | inserts `Approach { position: target_position, radius: size }` |
| `BehaviorFlee` | inserts `Approach` away from `from` by `ai_flee_distance` |
| `BehaviorPatrol` | `Approach { around_position, radius }` |
| `BehaviorGuard` / `BehaviorFollow` / `BehaviorKamikaze` | target valid → `Approach`; else `Idle` (previously `previous()` — see decision.md) |

`Idle`/`Chase` use `ship.min_range` (= `10000.0` seed lowered only by turret features)
and `ship.max_range` (= `1000.0` seed raised by turret feature ranges).

Skipped conditions: ships being followed (`CameraTarget.following`) or controlled
(`ControlledShip.controlling`) are skipped by AI.

The debug HUD (`update_ship_ui`, debug builds) reads the behavior marker via
`behavior_label(...)` and displays the behavior name without any controller enum.

Only teams 0 (player) and 1 are ever spawned. Schematics for AI ships come from
`load_saved_schematics()` and are spawned at mission-defined positions.

## Movement application

`PostUpdate::update_ship_behavior` converts `ShipBehavior` into `ExternalForce`:

- `Approach`: computes angle error between `forward` and direction-to-target; applies
  torque (clamped `turn_speed`) with a squared torque multiplier on angular velocity;
  thrusts forward if `angle_diff < π/3` and `distance > radius`; applies deceleration
  model `stop_distance = v² / (2 * LINEAR_DAMPING)`; zeroes force when inside it.
- `GoAround`: steers to a ring point at `radius`, shifted 45° forward to orbit; similar
  torque logic; forces away from target when inside ring.

`ShipBehavior` is an `Approach`/`GoAround { position, radius }` component written by AI
behavior systems. It is never removed intentionally; overwritten on next transition.

Player-controlled ships skip `update_ship_behavior`.

## Camera assist

`PlayerControlShip` handles scroll-wheel zoom (`MIN_ZOOM = 0.2`, `MAX_ZOOM = 30.0`) and
WASD/arrow camera pan (with Shift boost ×3). When `CameraTarget.following` is set, the camera
smoothly lerps toward the followed ship's core position
(`current.lerp(target, 1 - exp(-10 * dt))`).

## Selection

`SelectablePlugin`: `Ctrl`+`Left-drag` selects player ships within the drag box;
`SelectableState::Selecting` tracks start pos; selection takes entity under the box via
`BoundingBox` test + translation test, and emits `EntitySelectEvent { Vec<Entity> }`.

Consumers: Game HUD `update_select_panel` dumps `"Selected: <n>"` into the
selection panel.
