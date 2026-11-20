## 1. Ship-aware arrival controller

- [x] 1.1 Add `&ShipStats` to the `steer_ships` query in `core/src/content/ship/ai/steer.rs` and destructure it in the loop
- [x] 1.2 Rewrite `steer_arrive()` as a stateless speed-target controller: keep the `dist <= arrival_radius` zero-thrust early return, compute `safe = (2.0 * decel.max(0.1) * (dist - arrival_radius)).sqrt()`, `desired = safe.min(decel / damping.max(0.1))`, `thrust = ((desired - vel.dot(dir)) / decel.max(0.1)).clamp(-1.0, 1.0)`, and return `Steering { desired_heading: None, thrust_world: dir * thrust, throttle_cap: 1.0 }`; remove the `throttle_cap` parameter and the `mag` clamp
- [x] 1.3 Change the `Regroup | Investigate | Search` call site (steer.rs:155) to pass `stats.thrust_power / stats.mass.max(1.0)` and `game_cfg.0.linear_damping`
- [x] 1.4 Add a `decel` parameter to `steer_support()` and forward it from `steer_ships`, replacing its `support_throttle_cap` argument and `damping` use in the `steer_arrive()` call (steer.rs:350)

## 2. Remove dead throttle-cap config

- [x] 2.1 Remove `arrive_throttle_cap` and `support_throttle_cap` fields and defaults from `CombatTuning` in `core/src/content/ship/ai/config/types.rs`
- [x] 2.2 Remove the `arrive_throttle_cap: 0.85` and `support_throttle_cap: 0.9` keys from `core/assets/config/base/ai.ron`
- [x] 2.3 Verify no remaining references to either key exist in `core/src` and that AI config deserialization tests still pass

## 3. Tests

- [x] 3.1 Add unit tests for `steer_arrive` covering: inside arrival radius returns zero thrust and cap `0.0` with no `NaN`; a high-decel ship gets a higher desired speed than a low-decel ship at the same distance; overspeed produces negative thrust; thrust changes continuously across the braking boundary (no phase chatter)
- [x] 3.2 Add a regression test asserting that two ships with different `ShipStats.thrust_power / mass` receive different arrival `Steering.thrust_world` magnitudes in `Tactic::Search`
- [x] 3.3 Run targeted tests (`cargo test` for the AI module) and confirm no errors

## 4. Documentation

- [x] 4.1 Update `/docs/ai.md` steering section (around lines 46–49) to describe the ship-aware arrival controller and removal of non-combat throttle caps
- [x] 4.2 Update `/docs/balance.md` Mobility section and validation to state that the acceleration ratio is observable in Investigate/Search, not only in combat; keep the >5× requirement and the existing mass/thrust table unchanged
- [x] 4.3 Update `/docs/simulation.md` movement paragraph (lines 135–140) to describe arrival braking and per-ship cruise speed

## 5. Verification

- [x] 5.1 Run the game with `cargo dev` in the background and visually confirm ships move at noticeably different speeds in Investigate/Search and brake smoothly near waypoints (terminate the background process afterwards)
- [x] 5.2 Confirm no changes were made to `core/assets/config/base/block.ron` mass or force values or to `game.ron` thrust factors (the cancelled RON rebalance)
