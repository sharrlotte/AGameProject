## Why

The current power model draws every consumer's full rate permanently, splits a deficit with an equal-share rule that wastes surplus power, applies the shortage penalty twice, and overwrites efficiency buffs on every powered block. There is no energy storage, turrets either fire at full rate or not at all, and four overlapping efficiency concepts (`PowerEfficiency`, `BlockEfficiency`, `EfficiencyEffect`, `ShipStats.efficiency`) violate the single-source-of-truth rule. The power system must be rebuilt so that power is generated, stored and consumed only by active blocks, with a shortage degrading performance proportionally and overdrive blocks boosting nearby blocks.

## What Changes

- **BREAKING** Replace the equal-share solver with a per-ship grid solver: generation → booster priority → proportional split of the remainder across active consumers → battery charge/discharge.
- **BREAKING** Remove `PowerProducer`, `EnergyConsumer`, `EfficiencyProvider`, `PowerEfficiency`, `BlockEfficiency`, `EfficiencyEffect` and `ShipStats.efficiency`/`power_used`. Replace them with two runtime values per block: `Supply` (0..1, fraction of requested power received) and `Boost` (≥ 1, overdrive multiplier).
- Add block features `Generator { output }`, `Battery { capacity }`, `Consumer { rate }`, `Booster { radius, boost, rate }`. Cargo `Storage` is unchanged.
- Add a per-ship `Throttle { linear, angular }` (0..1) written by both player control and AI behaviour; thrusters only draw power while throttle is non-zero.
- Thrusters: draw `rate × max(linear, angular) × boost`; force and torque scale with `supply × boost`.
- Turrets gain an optional `power` sub-config:
  - Shot patterns (`Single`, `Burst`): internal battery `{ capacity, charge_rate }`; a full battery pays for one shot or one whole burst; firing requires a full battery and an elapsed `fire_rate` cooldown; charge speed scales with `supply × boost`; cooldown is divided by `boost`; a full battery draws nothing.
  - Beam patterns (`LaserBeam`, `PullingBeam`): continuous `{ drain }` while firing; damage/pull strength scale with `supply`; the beam stops below 0.01 supply; cooldown is divided by `boost`.
  - Ammo and power remain independent per turret.
- Boosters are binary (fully powered → boost, otherwise none), take priority power closest-to-core first, affect own-team blocks in a physics radius on any ship, strongest boost wins; boosted blocks draw extra power from their own ship. Overdrive applies to thrusters, turret reload/charge and generator output (generators use the previous tick's boost).
- Batteries charge/discharge instantly from surplus/deficit, spawn full, and lose their charge when destroyed. Turret internal batteries spawn full and are not part of grid storage.
- `ShipStats` exposes `power_production`, `power_demand`, `power_supply`, `power_stored`, `power_capacity`.
- Update `block.ron` and docs (`simulation.md`, `architecture.md`, `combat.md`, `config.md`, `defects.md`).

## Capabilities

### New Capabilities
- `power-grid`: per-ship grid solver, generators, batteries, supply ratio, ship power stats and tick order.
- `power-consumer`: active-only consumption, throttle-driven thruster demand and output scaling.
- `turret-power`: turret internal battery for shot patterns and continuous drain for beam patterns.
- `overdrive`: booster blocks, priority powering, boost application and stacking.
- `ship-throttle`: shared throttle intent written by player control and AI behaviour.

### Modified Capabilities
- `collapsed-features`: the thruster spawn scenario references the removed `PowerUsageComputed`; it must list the new power components.

## Impact

- Code: `game/core/src/content/block/feature/{energy,efficiency}` (replaced by `feature/power`), `feature/thruster`, `feature/turret`, `feature/component.rs`, `block/stat.rs`, `block/component.rs` (`stats_from_grid`), `bullet/system.rs` (shot and beam handlers), `ship/stat.rs`, `ship/system.rs` (player control and AI behaviour), `config/block.rs`, `simulation.rs`.
- Data: `game/core/assets/config/base/block.ron` (all power features renamed, turret `power` sub-configs for `laser_turret`, `meltdown`, `parallax`, removal of `EnergyConsumer` from ammo turrets `swarmer`/`spectre`, and new `battery` and `booster` block definitions).
- Docs: `docs/simulation.md`, `docs/architecture.md`, `docs/combat.md`, `docs/config.md`, `docs/defects.md`.
- Tests: existing energy solver tests are replaced.
