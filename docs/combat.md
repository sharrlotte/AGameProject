# Combat Spec — Turrets, Projectiles, Damage

## Turret model

`TurretComponent` (feature) spawns as child of a block and inserts:

- `Turret` — `range`, `turn_speed` (rad/s; `0.0` = fixed firing direction, no rotation),
  `barrels: Vec<Vec2>` (local px offsets rotated by turret transform), `strategy`,
  `shooting_pattern`.
- `TurretState` — `active: bool`, `target: Option<Entity>`, `shoot_at: Option<Vec2>`.
- `Radar::new(range, 1.0s)` — repeating sensor scan for enemy ships.
- `BlockEfficiency` (stats), `Sprite` (barrel/base image).

## Targeting (`FixedUpdate::update_turret_target`)

Teams gate logic: **player turrets** aim at the cursor (`Camera ↔ world-space`) when the
cursor is in range; the `GameVars.turret_control` resource selects behavior:

| Mode | Behavior |
|---|---|
| `Auto` (default) | no special cursor handling — fall through to auto-targeting |
| `Manual` | `shoot_at` = cursor position; `active` = left mouse held |
| `Hold` | same as Manual but only while LMB is held |

Nothing in the current build maps a key to `TurretControl::next()`, so the value is
always `Auto`.

**AI/auto turrets** scan `Radar.targets` (refreshed every 1.0 s). For each enemy ship the
first block's world position is used as the aim point. Selection:
`TargetingStrategy::Closest` (or identical placeholder implementations for `MostHP` and
`EnemyBullet`).

Turret `active` is set when aim error < `π/64` (fixed turrets) or `π/16` (rotating).

## Turret tracking (`turret_tracking`, FixedUpdate)

Rotates the turret transform toward `shoot_at` (player) or toward
`ship.core_position` (AI) at `turn_speed * delta_secs` per tick.

## Firing patterns

| Pattern | State component | Fire behavior |
|---|---|---|
| `Single` | `SingleShootPatternState { last_fired_at }` | Fires `barrels.len()` projectiles per `fire_rate` interval when `active` |
| `Burst` | `BurstShootPatternState` (`Shooting`/`Cooldown`) | Cycle through barrels, `delay` between shots, `amount` per burst, cooldown between bursts |
| `LaserBeam` | `LaserBeamPatternState { targets, beam, shooting }` | Continuous 5 s beam; deals `damage` per fixed tick inside the swath; `cooldown` between beams; beam sprite is a child entity |
| `PullingBeam` | `PullingBeamPatternState { beam }` | Component exists; **no systems registered** — inert |

`BurstShootPatternState` initial `last_fired_at = 0.0` — first burst fires immediately.
If the ammo `take()` fails, `current_amount` does not decrease — see defects.md.

## Projectile types

`ShootingProjectile` enum → spawned via `spawn_bullet` (bullet systems, `FixedUpdate`):

| Projectile | EntityType | Components | Movement | Notes |
|---|---|---|---|---|
| `Bullet` | `Bullet` | `Lifetime`, `Damage`, cuboid Sensor collider (`width/2 × length/2`), `CollisionGroups` | direct `Transform`, `velocity` (px/s) | `piercing > 0` keeps hitting; `Sensor` always set (redundant re-insert for pierce) |
| `Laser` | `Laser` | `Damage` | instant raycast from origin along turret forward; length clamped to `beam.length` | `Laser::Init` → `Laser::Fired { fired_at }`; visual sprite with width = ray length; fade after 0.25 s |
| `Missile` | `Missile` | child `MissileHitbox`, `Radar::new(lock_range, 1.0)` when untargeted | accelerating ease `powf(1.2)`, `t.clamp(0.4, 1.0)`; homing with `max_turn = 2.0 rad/s` | `accel_time = 0.35` fixed |
| Block spoiler | — | Missile hit children carry `Damage { blast_damage }` | explodes on collision into a `ball(blast_radius)` Sensor dealing `blast_damage` | |

`Bullet::range` (defaults from turret range) is the hard lifetime limit
(`destroy_out_of_range_bullet`); `Lifetime` is the fallback.

## Damage model

- `Damage` component on projectiles/hitboxes/blast circles.
- `handle_damage_collisions` (`FixedUpdate`) pairs `Damage` entities with `Health` entities
  on collision `Started` events and emits `DamageEvent`.
- `handle_damage_events` (`FixedPostUpdate`) applies `health -= damage` and spawns a red
  floating `Text2d` (z = 1,000,000) with lifetime 1.5 s.
- `handle_block_damage_event` (`FixedPostUpdate`) spawns a 0.1 s white
  `FLASH_OVERLAY_COLOR` flash overlay scaled to the block's pixel size.
- `Health ≤ 0` → block destruction (see simulation.md).

## Collisions

Bullets/missiles are **sensors** — no physics response, only events. Blocks are
**sensors**. The ship hull is the only non-sensor dynamic collider. `Group`/`Filter`
composition prevents friendly fire (`enemy_block_hitbox_filter` = all block-hitbox bits
except own team).

## Known-miss

- `Projectile::Laser` entities have no collider; they apply damage via `cast_ray` from
  origin per shot — corridor hit only, not swept along the beam.
- `handle_laser_beam` applies `DamageEvent` every fixed tick for the beam duration and
  spawns the damage text on every damage instance (health number spam).
- `cleanup_bullets` on `OnEnter(Home)` marks all `EntityType` entities `Death`.
