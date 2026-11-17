# Combat Spec — Turrets, Projectiles, Damage

## Turret model

`TurretComponent` (feature) spawns as child of a block and inserts:

- `Turret` — `range`, `turn_speed` (rad/s; `0.0` = fixed firing direction, no rotation),
  `barrels: Vec<Vec2>` (local px offsets rotated by turret transform), `strategy`,
  `shooting_pattern`.
- `TurretState` — `active: bool`, `target: Option<Entity>`, `shoot_at: Option<Vec2>`.
- `Radar::new(range, 1.0s)` — repeating sensor scan for enemy ships.
- `Boost` (stats), `Sprite` (barrel/base image).
- Optional `TurretPower` (`Charge { capacity, charge_rate }` for `Single`/`Burst` with `TurretCharge`; `Drain { rate }` for `LaserBeam`/`PullingBeam`), plus `Demand` and `Supply` when powered.
  Shot patterns require full `TurretCharge` and `fire_rate / boost` cooldown; beam damage/pull scale with `Supply` (stopping at `<= 0.01`), with cooldown divided by `boost`.

## Targeting (`FixedUpdate::update_turret_target`)

Both player and AI turrets auto-target:

**AI/auto turrets** scan `Radar.targets` (refreshed every 1.0 s). For each enemy ship the
first block's world position is used as the aim point. Selection:
`TargetingStrategy::Closest` (nearest by squared distance, ties broken by entity id),
`MostHP` (highest ship health, tie-broken by distance then id), or `EnemyBullet`
(interception-course aim via `lead_point` against projectiles, missiles inside `range/2`
preferred; aim point leads the target).

Turret `active` is set when aim error < `π/64` (fixed turrets) or `π/16` (rotating).

## Turret tracking (`turret_tracking`, FixedUpdate)

Rotates the turret transform toward the AI's chosen target at
`turn_speed * delta_secs` per tick.

## Firing patterns

| Pattern | State component | Fire behavior |
|---|---|---|
| `Single` | `SingleShootPatternState { last_fired_at }` | Fires `barrels.len()` projectiles per `fire_rate` interval when `active` |
| `Burst` | `BurstShootPatternState` (`Shooting`/`Cooldown`) | Cycle through barrels, `delay` between shots, `amount` per burst, cooldown between bursts |
| `LaserBeam` | `LaserBeamPatternState { targets, beam, shooting }` | Continuous 5 s beam; deals `damage` per fixed tick inside the swath; `cooldown` between beams; beam sprite is a child entity |
| `PullingBeam` | `PullingBeamPatternState { beam }` | Continuous beam for the turret's active window; pulls `Ship`-owned blocks toward the shooter via direct velocity nudge; no damage |

`BurstShootPatternState` initial `last_fired_at = 0.0` — first burst fires immediately.
If the ammo `take()` fails, `current_amount` transitions the pattern to `Cooldown` —
no stuck `Shooting` state.

## Projectile types

`ShootingProjectile` enum → spawned via `spawn_bullet` (bullet systems, `FixedUpdate`):

| Projectile | Components | Movement | Notes |
|---|---|---|---|
| `Bullet` | `Lifetime`, `Damage`, cuboid Sensor collider (`width/2 × length/2`), `CollisionGroups`, `DespawnOnExit(SceneState::Game)` | direct `Transform`, `velocity` (px/s) | `piercing > 0` keeps hitting; `Sensor` always set at spawn |
| `Laser` | `Damage`, `DespawnOnExit(SceneState::Game)` | instant raycast from origin along turret forward | `Laser::Init` → `Laser::Fired { fired_at }`; visual sprite with width = ray length; color fades linearly to black over `bullet_fade_time` (0.25 s) |
| `Missile` | child `MissileHitbox`, `Radar::new(lock_range, 1.0)` when untargeted, `DespawnOnExit(SceneState::Game)` | accelerating ease `powf(1.2)`, `t.clamp(0.4, 1.0)`; homing with `max_turn = 2.0 rad/s` | `accel_time = 0.35` |
| Block spoiler | — | Missile hit children carry `Damage { blast_damage }` | explodes on collision into a `ball(blast_radius)` Sensor dealing `blast_damage` | |

`Bullet::range` (defaults from turret range) is the hard lifetime limit
(`destroy_out_of_range_bullet`); `Lifetime` is the fallback.

## Damage model

- `Damage` component on projectiles/hitboxes/blast circles.
- `handle_damage_collisions` (`FixedUpdate`) pairs `Damage` entities with `Health` entities
  on collision `Started` events and emits `DamageEvent`.
- `handle_damage_events` (`FixedPostUpdate`) applies `health -= damage` and shows the
  damage number through the pooled `DamageTextPool` (64 `Text2d` slots spawned on
  `OnEnter(SceneState::Game)`; z = `LAYER_DAMAGE_TEXT`, lifetime `damage_number_lifetime`).
- `handle_block_damage_event` (`FixedPostUpdate`) spawns a 0.1 s white
  `FLASH_OVERLAY_COLOR` flash overlay scaled to the block's pixel size.
- `Health ≤ 0` → block destruction (see simulation.md).

## Collisions

Bullets/missiles are **sensors** — no physics response, only events. Blocks are
**sensors**. The ship hull is the only non-sensor dynamic collider. `Group`/`Filter`
composition prevents friendly fire (`enemy_block_hitbox_filter` = all block-hitbox bits
except own team).

## Permanent cleanup

- Everything combat-spawned carries `DespawnOnExit(SceneState::Game)`; Bevy despawns
  the tagged root and its children when the scene exits. There is no per-frame cleanup
  scan.

## Known-miss

- `Projectile::Laser` entities have no collider; they apply damage via `cast_ray` from
  origin per shot — corridor hit only, not swept along the beam.
- `handle_laser_beam` applies `DamageEvent` every fixed tick for the beam duration; damage
  text renders through the rotating 64-slot pool, so heavy beams recycle the same slots.
