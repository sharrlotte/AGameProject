## Context

`docs/balance.md` Goal 2 requires mobility to be inversely proportional to displacement and firepower, and its validation requires a >5× fastest-to-slowest acceleration ratio. That ratio is real in combat: `steer_engage()` uses `throttle_cap = 1.0` and preserves tangential thrust, so full-thrust acceleration applies. Outside combat, all movement funnels through `steer_arrive()`:

```rust
// current, steer.rs:360-393
let stop_dist = (speed * speed) / (2.0 * damping.max(0.1)); // global damping 0.8
if dist <= stop_dist { thrust_world = ZERO; throttle_cap = 0.0; }
let mag = ((dist - arrival_radius) / arrival_radius.max(100.0)).clamp(0.25, 1.0);
```

The braking test is mass-independent and quadratic in speed, so it imposes an absolute speed ceiling `v <= sqrt(2 × 0.8 × dist)` — roughly `62 px/s` at a 2400-unit search waypoint and `24 px/s` at the 350-unit arrival radius. Because the threshold grows with `speed²`, the ships with the highest acceleration trip it first and coast longest. Combined with the `mag` floor and the fixed `arrive_throttle_cap = 0.85`, the mobility advantage is actively compressed, matching the report that a 4-thruster Interceptor is barely faster than a 2-thruster B2 while investigating.

## Goals / Non-Goals

**Goals:**
- Make every AI ship travel at a speed derived from its own thrust-to-mass capability in `Regroup`, `Investigate`, `Search`, and `Support`.
- Preserve the spec's >5× mobility ratio outside combat so ship-size differences are observable in normal play.
- Brake with active reverse thrust so ships decelerate smoothly and do not overshoot waypoints.
- Keep the controller stateless and remove dead throttle-cap configuration.

**Non-Goals:**
- Changing any block mass, thrust force, or `thrust_power_factor` value (the cancelled RON rebalance).
- Changing combat steering (`steer_engage`, `steer_away_from`, posture logic) or player controls.
- Adding new RON knobs or new config sections.
- Adding dedicated arrival-tuning state per ship.

## Decisions

### 1. Ship-aware deceleration from live stats

Pass `decel = stats.thrust_power / stats.mass.max(1.0)` into `steer_arrive()`. `ShipStats.thrust_power` is the per-tick effective thrust (`base × supply × boost`) and `ShipStats.mass` is the same value given to Rapier's `ColliderMassProperties::Mass`, so `decel` is exactly the ship's real linear acceleration. `steer_ships()` adds `&ShipStats` to its query; `steer_support()` gains a `decel` parameter and forwards it.

**Rationale**: single source of truth — no second thrust/mass estimate, and boost/power starvation are reflected automatically. `ShipStats` is already recomputed every `FixedPreUpdate` tick.

### 2. Stateless speed-target controller replaces the coast branch and the `mag` clamp

```rust
fn steer_arrive(
    pos: Vec2,
    vel: Vec2,
    target: Vec2,
    arrival_radius: f32,
    decel: f32,
    damping: f32,
) -> Steering {
    let delta = target - pos;
    let dist = delta.length();
    if dist <= arrival_radius {
        return Steering { desired_heading: None, thrust_world: Vec2::ZERO, throttle_cap: 0.0 };
    }
    let dir = delta / dist;
    let safe = (2.0 * decel.max(0.1) * (dist - arrival_radius)).sqrt();
    let desired = safe.min(decel / damping.max(0.1));
    let thrust = ((desired - vel.dot(dir)) / decel.max(0.1)).clamp(-1.0, 1.0);
    Steering { desired_heading: None, thrust_world: dir * thrust, throttle_cap: 1.0 }
}
```

- `safe` is the speed from which the ship can still stop at the arrival boundary using its own `decel`.
- `desired` is capped at the ship's full-thrust terminal speed `decel / damping` (no cruise cap).
- The response is a fixed one-second time constant implicitly: thrust saturates when the speed error exceeds one second's worth of acceleration. No RON knob.
- A negative `thrust` is active reverse-thrust braking; `steer_support` and all AI ships are at level ≥ 4 (`omnidirectional_thrust_level`), so `update_ship_steering` applies the full vector.

**Rationale**: a continuous controller has no phase-switch chatter, needs no per-ship memory, and makes each ship's cruise speed proportional to its own capability, restoring the mobility gap outside combat.

### 3. Early return retained for the arrival boundary

`dist <= arrival_radius` still returns zero thrust and `throttle_cap = 0.0`. Besides ending the approach cleanly, this prevents `(dist - arrival_radius)` from going negative and producing `NaN` in `sqrt`.

### 4. Remove dead throttle-cap configuration

Delete `arrive_throttle_cap` and `support_throttle_cap` from `CombatTuning` (`types.rs`) and `ai.ron`. `steer_arrive()` no longer accepts a `throttle_cap` parameter and returns `1.0`. No replacement field is added.

**Rationale**: both keys existed only to feed the removed parameter; leaving them would be dead config. Adding neither a cruise-fraction nor a response knob keeps the change minimal and data-free.

### 5. Scope discipline — no stat rebalance

`block.ron` mass/force values and `game.ron` factors are untouched. The earlier decision to bump heavy-block masses and thruster force is explicitly cancelled because it would not address this behaviour.

## Risks / Trade-offs

- **Braking aggressiveness is derived, not tuned**: a one-second response is reasonable for all ships because `decel` scales the error, but there is no RON knob to soften it if ships feel too abrupt. Accepted per user decision to avoid new config.
- **Support ships now travel at their own full speed** rather than `0.9` cap; `Support` shares `steer_arrive`, so the removed cap applies there too. Accepted per user decision.
- **Low-skill ships (< 4)** would not apply reverse thrust because `update_ship_steering` projects thrust onto `local_y()`; current config guarantees all AI ships are level ≥ 4, and the player ship is skipped by `steer_ships`. If AI skill levels are ever configured below 4, reverse braking will not work for those ships.
- **Arrival overshoot**: the `safe` speed reaches zero at the arrival boundary, so the controller should asymptotically stop there; unit tests must confirm no oscillation at the boundary.
- **Docs drift elsewhere**: `config/game.rs:40` defaults `thrust_power_factor` to `50` while `game.ron` loads `100`, and `docs/simulation.md:140` says `linear_damping = 0.9` while RON loads `0.8`. This change does not fix those; they remain out of scope.

## Open Questions

None. All design decisions were resolved with the user before this proposal was created.
