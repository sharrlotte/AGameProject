# Combat Balance Spec — Durability, Mobility, Weapon Roles

## Goals

1. Ship loss is an attrition event, not a single hit: no weapon may destroy a `core`
   block in one shot, and the `core` is the last block a ship can lose.
2. Mobility is inversely proportional to displacement and firepower: small, lightly
   armed ships accelerate and turn faster than capital ships. A capital ship's
   long-range weapon is offset by an inability to track or escape a closing skirmisher.
3. Block cost is a presentation-only value and is excluded from combat tuning.

## Invariants

- `core` is the only `block_group: "core"` block; exactly one per valid ship. Ship
  death is defined by `core_of(grid)` returning `None`, not by aggregate HP.
- Ship HP is the sum of per-block HP and is used for HP-ratio AI/UI only. Damage is
  applied to the hit block, never to the aggregate.
- Linear acceleration is `Σ(thruster.force × thrust_power_factor × supply × boost) / Σ(block.mass)`.
  Angular acceleration is `Σ(thruster.torque × turn_speed_factor × supply × boost) / inertia`.
  There is no hard speed cap; steady-state speed follows `linear_damping`.
- `fire_rate` is the cooldown in seconds between shots/bursts, not shots per second.
- Triangles keep their documented 60% mass/health/cost/drop ratio relative to a
  same-cell-count `hull` block.
- AI role classification reads features, not `block_group`; `block_group` only
  orders registry display.

## Durability

Ship survivability comes from the `core` HP pool, with hull blocks absorbing hits
before the core is exposed. The core holds 15000 HP; no single projectile deals
more than 1200 damage, so a kill requires at least 13 core hits. Structural blocks
carry 1300-6000 HP, capital turrets 7000-16000 HP, and support blocks 3000-6000 HP.

| Block | HP | Block | HP |
|---|---:|---|---:|
| hull | 2200 | core | 15000 |
| wedge_1x1 / cap_1x1 | 1300 | reactor | 5000 |
| fin_1x2 | 2600 | laser_turret / swarmer | 3500 |
| spike_1x3 | 4000 | spectre / meltdown | 7000 |
| plate_2x2 | 6000 | foreshadow | 16000 |
| thruster_small | 6000 | colaris / parallax | 14000 |
| block_scrap | 4000 | shield_circle | 5000 |
| block_steel | 5000 | shield_plane | 4000 |
| battery | 3000 | booster | 4000 |

Barrier HP is separate from block HP: `shield_circle` 3500, `shield_plane` 2800.
Barriers absorb the full hit (no bleed-through) and regenerate after cooldown.

## Mobility

Light blocks reduce the denominator of the acceleration formula; capital blocks
increase it. Thruster output and power demand are unchanged. Every ship retains
the baseline 200 force / 60 torque from its core.

| Lighter mass | Value | Heavier mass | Value |
|---|---:|---|---:|
| thruster_small | 6 | plate_2x2 | 40 |
| core | 40 | block_scrap | 14 |
| reactor | 22 | block_steel | 16 |
| laser_turret | 6 | swarmer | 12 |
| battery | 10 | spectre | 70 |
| booster | 12 | meltdown | 80 |
| hull / wedge / cap / fin / spike | 10 / 6 / 6 / 12 / 18 | colaris / parallax | 130 / 110 |
| | | shield_circle / shield_plane | 80 / 65 |
| | | foreshadow | 200 |

Resulting acceleration (`thrust / mass`, normalized units) at spawn loadout:

| Design | Mass | Accel | Turn | Range | DPS | Total HP |
|---|---:|---:|---:|---:|---:|---:|
| Blackbird | 192 | 1146 | 30938 | 5000 | 120 | 71600 |
| Vanguard | 271 | 812 | 21919 | 5000 | 352 | 82200 |
| Interceptor | 386 | 570 | 15389 | 5000 | 360 | 119500 |
| Raider | 496 | 444 | 11976 | 5000 | 168 | 139700 |
| Aegis | 412 | 413 | 11141 | 6000 | 349 | 91200 |
| B2 | 1118 | 107 | 2898 | 15000 | 318 | 187800 |

The fastest scout is 10.7x more accelerative than the slowest capital ship. The AI arrival controller uses this same `thrust / mass` value as the ship's deceleration, so the ratio is observable during `Regroup`, `Investigate`, `Search`, and `Support` as well as in `Engage`: each ship cruises at its own `decel / linear_damping` speed and brakes with reverse thrust near a waypoint, with no separate non-combat throttle cap.

## Weapons

Damage per shot is set so no weapon exceeds 8% of core HP (1200 / 15000). Sustained
DPS scales with cooldown and burst size; `piercing` multiplies hits along a line and
is therefore limited on the highest-damage projectile.

| Turret | Damage | Cooldown | Burst / barrels | Piercing | Range |
|---|---:|---:|---|---:|---:|
| laser_turret | 60 (Laser) | 3.0 | 6 / 1 | 0 | 5000 |
| swarmer | 16 + 40 blast (Missile) | 2.0 | 1 / 2 | 0 | 5000 |
| colaris | 40 (Bullet) | 2.0 | 12 / 1 (delay 0.2) | 0 | 6000 |
| spectre | 28 (Bullet) | 0.2 | 12 / 2 (delay 0.2) | 2 | 5000 |
| foreshadow | 1200 (Bullet) | 12.0 | 1 / 1 | 10 | 15000 |
| meltdown | 2 per fixed tick (LaserBeam) | 10.0 (5s duration) | continuous | 0 | 3000 |
| parallax | none (PullingBeam) | continuous | continuous | 0 | 6000 |

Meltdown damage is per fixed tick (approximately 64 Hz), giving roughly 128 DPS at
full supply. PullingBeam applies a velocity impulse only and cannot destroy a block.

## Ship archetypes — trade-off matrix

| Build | Strength | Cost |
|---|---|---|
| Blackbird (scout) | Highest accel/turn, 192 mass, 4 thrusters | Single laser (120 DPS), lowest HP, no shield |
| Interceptor (gunship) | Best sustained DPS of the escorts (360), 3 lasers | 39 blocks, no shield, 5000 range |
| Raider (missile boat) | 139700 HP, 3 missile launchers | Lowest DPS (168), ammo-dependent, slow |
| Vanguard (assault) | 812 accel, 352 DPS, directional barrier | 82200 HP, no booster, 5000 range |
| Aegis (shield bruiser) | Circle shield, 349 DPS, booster | Thin hull, 412 mass, shield-dependent |
| B2 (dreadnought) | 15000 range, 2 colaris + railgun, 187800 HP, plate armor | 107 accel, 2 thrusters, 0 turn tracking on railgun, defenseless when closed |

The intended counter is positional: a capital ship wins only while the range
advantage holds. Once a skirmisher closes inside the fixed railgun's aim tolerance,
the capital ship cannot rotate its spine fast enough to respond and cannot outrun
the attacker. Structural HP keeps that engagement long rather than instantaneous.

## Validation

- `shipped_block_config_parses_and_bakes` in `config/block.rs` must pass.
- No block may have `health <= 0`, `mass <= 0`, or a negative weapon cooldown.
- Every triangle block's mass/health must remain at 60% of its same-cell-count hull
  baseline (1 cell: 6/1300, 2 cells: 12/2600, 3 cells: 18/4000).
- Highest single-hit damage (1200) must stay below 10% of core HP (1500).
- Fastest-to-slowest acceleration ratio must stay above 5x for the starter fleet, observable in both combat and AI arrival states.
