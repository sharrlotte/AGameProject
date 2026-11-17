## 1. Power module foundation

- [x] 1.1 Create `content/block/feature/power/` (`mod.rs`, `component.rs`, `grid.rs`, `booster.rs`) with `PowerPlugin` and chained `PowerSet::{Generate, Boost, Demand, Solve}` in `FixedFirst` / `SimulationSet::Stats`
- [x] 1.2 Add components `Supply`, `Boost`, `Demand`, `Generator`, `Battery`, `Charge`, `Consumer`, `Booster`, `BoosterState` and the named starvation constant
- [x] 1.3 Add `Feature::{Generator, Battery, Consumer, Booster}` with insert paths (consumers get `Demand` + `Supply`; boostable blocks get `Boost`; batteries spawn full)
- [x] 1.4 Add `FeatureConfig` variants and load-time validation (rate/radius/capacity ≥ 0, boost ≥ 1) in `config/block.rs`

## 2. Throttle

- [x] 2.1 Add `Throttle { linear, angular }` in `content/ship/throttle.rs`, inserted on every ship spawn path
- [x] 2.2 Player control writes throttle from input; FreeLook writes 0
- [x] 2.3 AI behaviours (`Approach`, `GoAround`) write throttle from applied force/torque multipliers

## 3. Grid solver

- [x] 3.1 Implement pure solver function (generation, booster priority list, demand, stored, capacity, dt → booster powered flags, supply, new stored)
- [x] 3.2 Generation system: Σ `Generator.output × Boost` (previous tick) per ship
- [x] 3.3 Booster priority system: sort by distance to ship core, power while available ≥ rate, write `BoosterState`
- [x] 3.4 Boost apply system: reset `Boost` to 1, rapier query per powered booster, own-team filter, max stacking
- [x] 3.5 Solve system: proportional supply, battery charge/discharge distributed proportional to capacity, write `Supply`, `Charge`, `ShipStats`
- [x] 3.6 Unit tests for the solver covering every `power-grid` and `overdrive` scenario

## 4. Thrusters

- [x] 4.1 Thruster demand system: `rate × max(linear, angular) × Boost`
- [x] 4.2 Update `make_stat_system!` to compute `base × supply × boost` from `Supply`/`Boost` (supply 1 when absent), keep starvation disable
- [x] 4.3 Tests for `power-consumer` scenarios

## 5. Turret power

- [x] 5.1 Add `TurretPowerConfig::{Charge, Drain}` to `TurretComponentConfig.power`; reject mismatched pattern with warning
- [x] 5.2 Add `TurretPower` + `TurretCharge` (spawns full) on turret insert
- [x] 5.3 Turret demand system: charge turrets `charge_rate × Boost` when not full; beams `rate` while active
- [x] 5.4 Charge advance in `Weapons`: `charge_rate × supply × boost × dt`, clamped
- [x] 5.5 Single/Burst handlers: require full charge and `fire_rate / boost` cooldown; reset charge on fire (once per burst); remove `0.999` gate
- [x] 5.6 LaserBeam/PullingBeam handlers: damage and pull × supply, stop at starvation, cooldown / boost
- [x] 5.7 Tests for `turret-power` scenarios

## 6. Remove legacy power and efficiency

- [x] 6.1 Delete `feature/energy` and `feature/efficiency` modules and their plugins
- [x] 6.2 Remove `PowerEfficiency`, `BlockEfficiency`, `EfficiencyEffect` uses from thruster, turret, stat macro, bullet systems
- [x] 6.3 `ShipStats`: replace `power_used`/`efficiency` with `power_supply`, `power_stored`, `power_capacity`; update `stats_from_grid` aggregation and HUD/readers
- [x] 6.4 Update editor spawn path to the new component layout

## 7. Data

- [x] 7.1 Rename features in `block.ron` (`PowerProducer` → `Generator` on `core`/`reactor`, `EnergyConsumer` → `Consumer` on `thruster_small`)
- [x] 7.2 Set turret `power` in `block.ron`: `laser_turret` (`Charge(capacity: 15.0, charge_rate: 5.0)`), `meltdown` (`Drain(rate: 10.0)`), `parallax` (`Drain(rate: 6.0)`), and `None` on `swarmer`, `spectre`, `colaris`, `foreshadow` (removing old `EnergyConsumer` entries)
- [x] 7.3 Add `battery` (`capacity: 100.0`) and `booster` (`radius: 256.0, boost: 1.5, rate: 8.0`) block definitions to `block.ron`

## 8. Docs and verification

- [x] 8.1 Update `docs/simulation.md` power section to the new grid spec
- [x] 8.2 Update `docs/architecture.md` scheduling table and module tree; update `simulation.rs` doc table
- [x] 8.3 Update `docs/combat.md` (turret power), `docs/config.md` (new features), `docs/defects.md` (defects 1–8 resolved)
- [x] 8.4 `cargo check` for the core crate, run power/turret/thruster tests, then `cargo dev` and verify ship movement and turret firing via BRP
