## Why

Players observe that a 4-thruster Interceptor is barely faster than a 2-thruster B2 outside combat, and that ships crawl at roughly one-third of their achievable speed while in Investigate/Search. The mobility spec (`docs/balance.md`) declares that small ships must accelerate and turn faster than capital ships and validates a >5× fastest-to-slowest acceleration ratio, but that advantage only exists at full thrust in `Engage`.

The cause is in AI steering, not ship stats. `steer_arrive()` (`core/src/content/ship/ai/steer.rs`) governs `Regroup`, `Investigate`, `Search`, and `Support`. It derives a braking distance from the **global** `linear_damping` (`0.8`) and zeroes thrust whenever `dist <= speed² / (2 × damping)`. This creates a mass-independent speed ceiling (about `62 px/s` at a 2400-unit search waypoint), and because the threshold grows with `speed²`, faster ships trip the brake sooner and spend more of the trip coasting. The remaining thrust is scaled by fixed `mag` and throttle-cap multipliers, so every ship is suppressed to the same crawl and the size-based mobility advantage cannot express itself.

A previous RON rebalance of block mass and thrust force was considered and **explicitly cancelled**: it would change combat magnitudes only and leave this behaviour untouched.

## What Changes

- Rewrite `steer_arrive()` as a stateless, ship-aware speed-target controller. Pass each ship's own deceleration `decel = ShipStats.thrust_power / ShipStats.mass.max(1.0)` (and the configured `linear_damping`) into the function instead of using a global constant.
- Compute the safe approach speed from the ship's own `decel` and the remaining distance to the arrival boundary; drive thrust proportionally to the speed error so the ship brakes with active reverse thrust and cruises at its own achievable speed (`decel / damping`) rather than a fixed crawl.
- Remove the `throttle_cap` parameter from `steer_arrive()`; the controller encodes magnitude in `thrust_world` and returns `throttle_cap: 1.0`.
- Delete the now-unused `arrive_throttle_cap` (`0.85`) and `support_throttle_cap` (`0.9`) keys from `core/assets/config/base/ai.ron` and their fields from `AiConfig`'s `CombatTuning` (`core/src/content/ship/ai/config/types.rs`). Add no new RON knobs.
- Thread `decel` through `steer_support()` into `steer_arrive()`, and read `ShipStats` in `steer_ships()`.
- Update `/docs/ai.md`, `/docs/balance.md`, and `/docs/simulation.md` to reflect the new arrival-steering behaviour. Do not change any mass or thrust values in `block.ron` or `game.ron`.

## Capabilities

### New Capabilities
- `ai-arrival-steering`: Ship-aware arrival steering for `Regroup` / `Investigate` / `Search` / `Support` — a stateless speed-target controller using each ship's own thrust-to-mass deceleration, with active reverse-thrust braking and no fixed throttle cap.

### Modified Capabilities
- `smart-utility-ai`: `steer_ships` / `steer_support` now consume `ShipStats` and the shared arrival controller; non-combat throttle-cap configuration is removed.
- `config-validation`: `CombatTuning` loses `arrive_throttle_cap` and `support_throttle_cap`.

## Impact

- **Dependencies**: None new. Uses existing `ShipStats`, `GameConfigValue.linear_damping`, and `Steering`.
- **Systems & Modules**:
  - `core/src/content/ship/ai/steer.rs`: add `&ShipStats` to the `steer_ships` query, pass `decel` at the `Regroup | Investigate | Search` and `Support` call sites, rewrite `steer_arrive()` and `steer_support()` signatures.
  - `core/src/content/ship/ai/config/types.rs`: remove the two `CombatTuning` fields and their defaults.
  - `core/assets/config/base/ai.ron`: remove the two keys.
- **Documentation**: Updates `docs/ai.md`, `docs/balance.md`, `docs/simulation.md`.
- **No config migration**: Existing save files and starter designs are unaffected; only two AI tuning keys are removed.
