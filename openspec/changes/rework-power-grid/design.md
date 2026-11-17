## Context

Power currently lives in two modules:

- `feature/energy`: `PowerProducer`, `EnergyConsumer`, `PowerEfficiency`, `update_power_efficiency` (equal-share solver, `FixedFirst` / `SimulationSet::Stats`).
- `feature/efficiency`: `BlockEfficiency`, `EfficiencyEffect`, `EfficiencyProvider`, `update_block_efficiency_base` + `apply_efficiency_buffs` (`EfficiencyUpdateSet`).

Defects found during review:

| # | Defect |
|---|---|
| 1 | Consumer demand is constant; idle thrusters and target-less turrets draw full rate. |
| 2 | `share = production / n` wastes surplus when small consumers are over-fed. |
| 3 | `BlockEfficiency.base` = last tick's `ShipStats.efficiency`, then multiplied by the block's own ratio → shortage applied twice, unpowered blocks penalised. |
| 4 | `apply_power` overwrites the provider buff on every powered block. |
| 5 | Producers carry no efficiency → generation cannot be boosted. |
| 6 | Turrets are gated at `PowerEfficiency ≥ 0.999`; efficiency never affects reload. |
| 7 | No energy storage exists. |
| 8 | Four overlapping efficiency concepts. |

Thrust is aggregated per ship (`ShipStats.thrust_power`, `rotate_speed`) and applied by `ship/system.rs` from player keys (binary) and AI behaviours (`force_multiplier`, `torque_multiplier`).

Pipeline (from `simulation.rs`): `FixedFirst/Stats` → `FixedPreUpdate/Stats` → `FixedUpdate/{Ai, Control, Weapons}` → `FixedPostUpdate/{Damage, Death}`.

## Goals / Non-Goals

**Goals:**
- One per-ship grid solver with generation, priority boosters, proportional consumers and instant batteries.
- Exactly two per-block runtime power values: `Supply` and `Boost`.
- Demand only from active blocks, driven by a shared `Throttle` for thrusters and battery state for turrets.
- Overdrive on thrusters, turret reload/charge, generator output.
- All tunables in `block.ron`; hot reload keeps working.

**Non-Goals:**
- Directional (per-facing) thrusters.
- Conduits / multiple grids per ship.
- Fuel or item-consuming generators (design leaves room only).
- Battery charge/discharge rate limits.
- Power HUD (stats are exposed; UI is a later change).
- Overdrive on battery rates, turret damage or range.

## Decisions

### D1. Module layout

`feature/energy` and `feature/efficiency` are deleted and replaced by one feature folder:

```
content/block/feature/power/
  mod.rs        PowerPlugin, PowerSet (ordered sub-sets)
  component.rs  Supply, Boost, Demand, Generator, Battery, Charge, Consumer, Booster, BoosterState
  grid.rs       per-ship solver (pure function over totals + system wrapper)
  booster.rs    booster priority + boost application systems
content/ship/throttle.rs   Throttle component
```

Turret power lives in `feature/turret` (`TurretPower`, `TurretCharge`) because it is turret-specific behaviour; it only reads `Supply`/`Boost` and writes `Demand`.

*Alternative:* keep `energy` and `efficiency` folders. Rejected: the two concepts are now one pipeline and the split caused defects 3 and 4.

### D2. Runtime values

| Component | Range | Writer | Readers |
|---|---|---|---|
| `Demand { value }` | ≥ 0 power/s | per-feature demand systems | grid solver |
| `Supply { value }` | 0..1 | grid solver | thruster stat, turret charge, beam |
| `Boost { value }` | ≥ 1 | booster apply system | demand systems, thruster stat, turret, generator |
| `Charge { value }` | 0..capacity | grid solver | ship stats |
| `BoosterState { powered }` | bool | booster priority | booster apply |

Every block that can consume power gets `Demand` + `Supply`; every block that can be boosted (thruster, turret, generator) gets `Boost`. `Boost` resets to 1 each tick before application.

*Alternative:* one combined `efficiency = supply × boost`. Rejected: demand needs `boost` without `supply`, turrets need them separately (charge vs cooldown), and it recreates defect 4.

### D3. Tick order

All in `FixedFirst`, `SimulationSet::Stats`, chained via `PowerSet` sub-sets:

```
PowerSet::Generate   generation = Σ Generator.output × Boost (previous tick)
PowerSet::Boost      booster priority (closest to core first) → BoosterState
                     reset Boost = 1 → apply Boost (max of powered boosters in radius, own team)
PowerSet::Demand     thruster demand  = rate × max(linear, angular) × Boost
                     turret demand    = charge_rate × Boost   (battery not full, else 0)
                     beam demand      = drain                 (while firing, else 0)
PowerSet::Solve      proportional split + battery charge/discharge → Supply, Charge, ShipStats
```

Then `FixedPreUpdate/Stats`: thrust/rotate stats read `Supply` × `Boost`.
Then `FixedUpdate/Weapons`: turret charge advance, shot/beam handlers.

Generators read last tick's `Boost` because generation must be known before boosters are decided (avoids circularity). Consumer demand uses this tick's `Boost`.

### D4. Grid solver

Per ship, `available = generation + stored`:

```
1. Boosters (sorted by distance to core, ascending; tie → Entity order):
     if available ≥ booster.rate → powered, available -= rate
     else → unpowered (no boost, draws nothing)
2. demand = Σ Demand of non-booster blocks
   supply = demand == 0 ? 1 : clamp(available / demand, 0, 1)
3. net = generation − booster_draw − demand × supply
     net ≥ 0 → stored = min(stored + net × dt, capacity)
     net < 0 → stored = max(stored + net × dt, 0)
4. Supply on every consumer = supply; ShipStats updated.
```

Values are per second; battery change uses fixed `dt`. `available` treats stored energy as instantly dischargeable (no rate limit), converted to power for the tick as `stored / dt`.

Battery charge is distributed over battery blocks proportional to capacity so losing a block loses its share. Solver core is a pure function over totals for unit tests.

*Alternative:* priority tiers for all consumers. Rejected by user; only boosters get priority.

### D5. Throttle

`Throttle { linear: f32, angular: f32 }` on every ship, both clamped 0..1.

- Player control (`Control` set): `linear = 1` if any of W/S/Q/E held, `angular = 1` if A/D held, else 0; FreeLook writes 0.
- AI behaviour (`Ai` set): `linear = force_multiplier` when a forward force is applied else 0; `angular = torque_multiplier` when torque is non-zero else 0.

Because `Stats` runs before `Ai`/`Control`, demand uses the previous tick's throttle (one fixed tick lag, ~16 ms).

### D6. Thruster output

`ThrustPower = base × supply × boost`, same for `RotateSpeed`. `make_stat_system!` reads `Supply` and `Boost` instead of `BlockEfficiency`/`EfficiencyEffect`. Supply ≤ 0.01 disables the stat (existing behaviour). Thrusters without `Consumer` (e.g. Core) have supply 1.

### D7. Turret power

Config (inside `Turret` in `block.ron`):

```ron
power: Some(Charge((capacity: 20.0, charge_rate: 10.0)))   // Single / Burst
power: Some(Drain((rate: 6.0)))                           // LaserBeam / PullingBeam
power: None                                               // unpowered
```

Runtime:
- `TurretCharge { value }` spawns full.
- Shot: fire when `value ≥ capacity` AND `elapsed ≥ last_fired + fire_rate / boost`; on fire `value = 0` (one shot or whole burst). Each tick in `Weapons`: `value += charge_rate × supply × boost × dt`, clamped.
- Beam: demand = `rate` while the beam is active; damage and pull strength × `supply`; beam stops when `supply ≤ 0.01`; cooldown / `boost`.
- Pattern/power mismatch (e.g. `Drain` on `Single`) is rejected at config load with a warning and the turret spawns unpowered.

The `0.999` gate and `PowerEfficiency` reads in `bullet/system.rs` are removed.

### D8. Booster

`Booster { radius, boost, rate }`. Priority by distance from the booster to its ship's core block. Application uses the existing rapier `intersect_shape` with own-team filtering; a block in range of several powered boosters gets the max. A boosted block on an allied ship draws extra from its own ship's grid via its `Demand`.

### D9. Batteries

`Battery { capacity }` + `Charge` (spawns full). Destroyed battery → its charge is removed from the ship total. Turret charge is not grid storage.

### D10. Ship stats

`ShipStats` fields: `power_production`, `power_demand`, `power_supply`, `power_stored`, `power_capacity`. `power_used` and `efficiency` are removed. `stats_from_grid` aggregates structural values (production, capacity); runtime fields are written by the solver.

### D11. Starvation threshold

The 0.01 threshold stays hardcoded as one named constant in `feature/power` (user decision), shared by thrusters and beams.

## Risks / Trade-offs

- [Throttle lag of one fixed tick] → negligible at fixed rate; documented.
- [Generator boost lags one tick] → accepted to break the booster/generator cycle.
- [Booster binary with priority can starve all consumers when a booster takes the last power] → intended by priority rule; visible via `power_supply`.
- [Rapier query per booster per tick] → only booster count queries; boosters are rare blocks.
- [Instant battery discharge makes small batteries cover large spikes] → balance via capacity in `block.ron`.
- [Removing `BlockEfficiency` touches every thruster/turret spawn path and the editor] → covered by spawn tests and `collapsed-features` scenario update.

## Migration Plan

1. Land new `feature/power` + throttle alongside removal of old modules in one change (no compatibility layer).
2. Rename features in `block.ron` (`PowerProducer` → `Generator`, `EnergyConsumer` → `Consumer`, `EfficiencyProvider` → `Booster`), add turret `power` sub-configs.
3. Saves: power runtime state is not persisted; charges reset to full on load.

### D12. Block configuration (`block.ron`)

- Existing producers/consumers:
  - `core`: `Generator((output: 10.0))`
  - `reactor`: `Generator((output: 25.0))`
  - `thruster_small`: `Consumer((rate: 3.0))`
- Turrets:
  - `laser_turret`: `power: Some(Charge((capacity: 15.0, charge_rate: 5.0)))` (remove top-level `EnergyConsumer`)
  - `meltdown`: `power: Some(Drain((rate: 10.0)))`
  - `parallax`: `power: Some(Drain((rate: 6.0)))`
  - `swarmer`, `spectre`, `colaris`, `foreshadow`: `power: None` (remove top-level `EnergyConsumer` from `swarmer` and `spectre`)
- New blocks:
  - `battery`: 1×1, `block_group: "base"`, `mass: 15.0`, `health: 1500.0`, `cost: 100`, texture `"block/titanium-wall-large.png"`, `Battery((capacity: 100.0))`, drops `steel: 10`
  - `booster`: 1×1, `block_group: "base"`, `mass: 20.0`, `health: 2000.0`, `cost: 250`, texture `"block/shielded-wall.png"`, `Booster((radius: 256.0, boost: 1.5, rate: 8.0))`, drops `steel: 15`

## Open Questions

None — all design and configuration decisions have been approved.
