# enemy-threats Specification

## Purpose

Enemy threats: the authored archetype roster, the enemy brain state machine, stagger and steering
behavior, directional damage and armor, the death pipeline, tiered spawning, the Tank, Spitter, and
Exploder archetypes, ranged and detonation behaviors, enemy primitive presentation, and hot reload of
enemy configuration.

## Requirements

### Requirement: Authored enemy archetype roster

The simulation SHALL define every enemy archetype in `game/assets/config/enemies/enemies.ron` as a
uniquely keyed entry in `EnemiesConfig::archetypes`. Each entry MUST carry the archetype identity, combat
stats, `scrap_value`, a positive `pierce_cost`, a spawn policy, and an ordered behavior list.
`EnemiesConfig` MUST validate the roster before it is used and MUST reject duplicate kinds, a missing
Walker, non-positive or non-finite stats, a non-positive or non-finite `pierce_cost`, invalid behavior
values, and a roster with no positive spawn weight.

#### Scenario: Authored roster loads

- **WHEN** `enemies.ron` is parsed
- **THEN** the roster contains unique entries for Walker, Runner, Tank, Spitter, and Exploder with
  positive stats, positive piercing costs, and at least one positive spawn weight

#### Scenario: Duplicate archetype is rejected

- **WHEN** validation runs on a roster containing two entries with the same `EnemyKind`
- **THEN** validation fails with an error naming the duplicate kind

#### Scenario: Walker is required

- **WHEN** validation runs on a roster with no Walker entry
- **THEN** validation fails because Walker is the guaranteed spawn fallback

#### Scenario: Invalid piercing cost is rejected

- **WHEN** an archetype authors a zero, negative, or non-finite `pierce_cost`
- **THEN** validation fails with an error naming the archetype and field

#### Scenario: Invalid behavior parameters are rejected

- **WHEN** an archetype declares an Armor behavior with `arc_deg` outside `(0, 360]` or a negative
  multiplier, a Melee behavior with a negative damage, cooldown, wind-up, reach, or knockback value, a
  Ranged behavior with a negative damage, cooldown, wind-up, reach, or projectile speed value, an
  `arc_deg` outside `(0, 90)`, or a pool with a non-positive radius or lifetime, or a Detonate behavior
  with a non-positive trigger radius or radius or a negative fuse, damage, or knockback value
- **THEN** validation fails with an error naming the archetype and behavior

### Requirement: Enemy brain state machine

Every enemy SHALL carry an `EnemyBrain` with the states `Idle`, `Alert`, `Engage`, and `Stagger`. The
brain SHALL acquire a survivor target when one is within the configured aggro radius, SHALL remember the
last known target position when the target leaves the deaggro radius, and SHALL return to `Idle` after the
alert duration elapses.

#### Scenario: Proximity starts engagement

- **WHEN** an `Idle` enemy is within the configured aggro radius of a survivor
- **THEN** its brain enters `Engage` and records that survivor as its target

#### Scenario: Losing the target enters alert

- **WHEN** an `Engage` enemy has no survivor within the deaggro radius for the configured deaggro duration
- **THEN** its brain enters `Alert` and remembers the last known target position

#### Scenario: Alert expires to idle

- **WHEN** an `Alert` enemy has no target for the configured alert duration
- **THEN** its brain returns to `Idle` and clears its target

#### Scenario: Damage forces engagement

- **WHEN** an enemy receives damage from a survivor while outside its aggro radius
- **THEN** its brain enters `Engage` and targets the damage source

### Requirement: Stagger reaction

An enemy SHALL enter `Stagger` when its received knockback velocity exceeds the configured stagger
threshold, and MUST leave `Stagger` after the configured stagger duration. A staggered enemy MUST NOT
steer toward its target or start a melee or ranged wind-up. The Tank's mass-based knockback resistance
MUST make ordinary bullet hits insufficient to stagger it.

#### Scenario: Heavy knockback staggers

- **WHEN** an enemy receives a knockback impulse whose resulting velocity exceeds the stagger threshold
- **THEN** the enemy enters `Stagger` for the configured duration and stops moving

#### Scenario: Stagger expires

- **WHEN** the stagger duration elapses
- **THEN** the enemy returns to `Engage` if it still has a target, otherwise to `Alert`

#### Scenario: Tank resists ordinary bullets

- **WHEN** a Tank is hit by a standard rifle round
- **THEN** the resulting knockback velocity stays below the stagger threshold

### Requirement: Steering and separation

Enemies SHALL steer toward their target while in `Engage`, toward the last known position while in
`Alert`, and MUST NOT steer while in `Idle`, `Stagger`, an active melee or ranged wind-up, or while an
armed fuse roots the attacker. Steering MUST include a separation term that pushes overlapping enemies
apart without requiring a spatial grid.

#### Scenario: Engage chases the target

- **WHEN** an enemy is in `Engage` with a live target
- **THEN** its horizontal velocity points toward the target

#### Scenario: Wind-up roots the attacker

- **WHEN** an enemy is in an active melee or ranged wind-up
- **THEN** its horizontal velocity is zero for the duration of the wind-up

#### Scenario: Separation prevents stacking

- **WHEN** two enemies overlap and are both steering
- **THEN** their separation terms push them in opposing lateral directions

### Requirement: Directional damage and armor

`DamageMessage` SHALL carry `kind: DamageKind` and `hit_dir: Vec3`. Damage resolution SHALL apply an
`Armor` component when present: frontal damage uses `front_mult` when
`facing.dot(-hit_dir) >= cos(arc_deg / 2)`, back damage uses `back_mult`, and any kind listed in `bypass`
MUST ignore armor. A zero or non-finite `hit_dir` MUST be treated as frontal. Existing weapons MUST
default to `Ballistic` without changing their authored RON entries.

#### Scenario: Frontal ballistic damage is reduced

- **WHEN** a ballistic hit arrives from the side a Tank is facing
- **THEN** the damage is multiplied by the Tank's `front_mult`

#### Scenario: Back ballistic damage is not reduced

- **WHEN** a ballistic hit arrives from behind a Tank
- **THEN** the damage is multiplied by the Tank's `back_mult`

#### Scenario: Bypass kind ignores armor

- **WHEN** an explosive hit arrives on a Tank whose armor lists `Explosive` as a bypass kind
- **THEN** the full damage is applied

#### Scenario: Missing direction is frontal

- **WHEN** a damage message arrives with a zero `hit_dir`
- **THEN** armor treats the hit as frontal rather than panicking or granting a back-hit bonus

### Requirement: Enemy death pipeline

Lethal damage SHALL mark the enemy with `Dying`, emit `EnemyKilledMessage` exactly once with the
archetype's `scrap_value`, and MUST NOT despawn the entity during damage resolution. A `Dying` enemy MUST
be excluded from targeting and steering. Death effects SHALL resolve in `DeathResolutionSet` before a
cleanup pass despawns `Dying` entities.

#### Scenario: Lethal damage emits one kill message

- **WHEN** an enemy's health reaches zero
- **THEN** `EnemyKilledMessage` is emitted once with the enemy's kind and `scrap_value`

#### Scenario: Further damage does not duplicate the kill

- **WHEN** a `Dying` enemy receives additional damage before cleanup
- **THEN** no second `EnemyKilledMessage` is emitted

#### Scenario: Death effects run before despawn

- **WHEN** a death effect system queries `Dying` enemies in `DeathResolutionSet`
- **THEN** the enemy entity and its transform are still available during that system

#### Scenario: Dying enemies are ignored by AI

- **WHEN** an enemy is `Dying`
- **THEN** it is not selected as an AI target and does not steer or attack

### Requirement: Tiered spawn policy

The spawner SHALL select only archetypes whose `spawn.unlock_secs` is at or below the current run
elapsed time, SHALL weight the selection by the authored `spawn.weight`, and MUST respect both the global
active-enemy cap and each archetype's `spawn.max_active` cap. If every eligible archetype is at its cap,
no enemy is spawned for that tick.

#### Scenario: Locked archetype does not spawn

- **WHEN** elapsed run time is below an archetype's `unlock_secs`
- **THEN** that archetype is excluded from the spawn roll

#### Scenario: Unlocked archetype can spawn

- **WHEN** elapsed run time reaches an archetype's `unlock_secs`
- **THEN** that archetype becomes eligible for the weighted spawn roll

#### Scenario: Per-archetype cap is respected

- **WHEN** an archetype has reached its `spawn.max_active`
- **THEN** it is excluded from the spawn roll while other eligible archetypes may still spawn

#### Scenario: Global cap is respected

- **WHEN** the total active enemy count reaches the global maximum
- **THEN** no new enemy is spawned

### Requirement: Tank archetype

The Tank SHALL spawn with high health, slow movement, high mass, frontal ballistic armor, and a wind-up
melee slam. The slam MUST deal no damage before the wind-up completes, MUST deal damage only if the
target is still within reach when the wind-up completes, and MUST apply knockback to the survivor.
The Tank's authored `scrap_value` MUST be awarded through the death pipeline.

#### Scenario: Tank frontal armor mitigates bullets

- **WHEN** a Tank is shot from the front with a ballistic weapon
- **THEN** the incoming damage is reduced by the authored front multiplier

#### Scenario: Tank back is vulnerable

- **WHEN** a Tank is shot from behind with a ballistic weapon
- **THEN** the incoming damage uses the authored back multiplier

#### Scenario: Wind-up telegraphs the slam

- **WHEN** a Tank begins a melee attack
- **THEN** it roots in place for the authored wind-up duration and deals no damage during that time

#### Scenario: Slam resolves only in reach

- **WHEN** the wind-up completes and the survivor is still within melee reach
- **THEN** the Tank deals the authored melee damage and knockback

#### Scenario: Missed slam deals no damage

- **WHEN** the wind-up completes and the survivor has left melee reach
- **THEN** the Tank deals no damage and starts its cooldown

#### Scenario: Tank awards authored scrap

- **WHEN** a Tank dies
- **THEN** the kill message carries the Tank's authored `scrap_value`

### Requirement: Enemy primitive presentation

Every implemented archetype SHALL render as a distinct primitive assembly under a rotating `VisualBody`
root. The Tank MUST read as armored through a larger silhouette, a front plate, shoulders, and a head.
Hit feedback MUST flash every mesh part of a multi-part enemy and MUST restore each part's authored
material afterward.

#### Scenario: Tank has an armored silhouette

- **WHEN** a Tank is spawned
- **THEN** its visual root contains a torso, front plate, shoulders, head, and arms

#### Scenario: Facing rotates the whole assembly

- **WHEN** an enemy changes facing
- **THEN** its visual root rotates and every child part follows

#### Scenario: Hit flash covers every part

- **WHEN** a multi-part enemy receives damage
- **THEN** every part switches to the hit-flash material and later restores its own base material

### Requirement: Enemy hot reload

Editing `enemies.ron` while the game is running SHALL update live enemy stats, behavior values, and
`pierce_cost`, and SHALL apply spawn policy and scrap changes to subsequent spawns and kills. A structural
change that adds or removes a behavior on an existing archetype MUST NOT panic; it SHALL log a warning and
take effect on the next deploy.

#### Scenario: Tank armor hot reloads

- **WHEN** `front_mult` changes in `enemies.ron` during a run
- **THEN** live Tanks use the new multiplier for subsequent hits

#### Scenario: Piercing cost hot reloads

- **WHEN** `pierce_cost` changes in `enemies.ron` during a run
- **THEN** live enemies of that archetype use the new cost for subsequent projectile hits

#### Scenario: Spawn policy hot reloads

- **WHEN** `unlock_secs`, `weight`, or `max_active` changes during a run
- **THEN** subsequent spawn rolls use the new policy

#### Scenario: Structural behavior change is safe

- **WHEN** a behavior is added to or removed from an archetype during a run
- **THEN** the game logs a warning and does not panic

### Requirement: Ranged attack behavior

An archetype with a `Ranged` behavior SHALL hold a preferred distance from its target, telegraph before
firing, and launch exactly one acid projectile per attack. The attack MUST root the attacker for the
wind-up and MUST NOT deal damage during it. The projectile MUST arc under gravity, MUST pass through
combatants, and MUST create a hazard pool where it first contacts world geometry or where its lifetime
expires. The cooldown MUST gate the next attack.

#### Scenario: Wind-up telegraphs the shot

- **WHEN** a ranged enemy starts an attack
- **THEN** it stops steering for the authored wind-up and no projectile exists yet

#### Scenario: Shot fires after the wind-up

- **WHEN** the wind-up completes and the target is still within the authored reach
- **THEN** the enemy launches one acid projectile toward the target and starts its cooldown

#### Scenario: Shot is cancelled when the target escapes

- **WHEN** the wind-up completes and the target is beyond the authored reach
- **THEN** no projectile is launched and the cooldown still starts

#### Scenario: Cooldown gates the next attack

- **WHEN** the enemy fires and the cooldown has not elapsed
- **THEN** it does not start another wind-up

#### Scenario: Preferred range is held

- **WHEN** an engaged ranged enemy is closer than the authored preferred distance
- **THEN** it backs away from the target instead of closing to contact

#### Scenario: Projectile ignores combatants

- **WHEN** an acid projectile passes through an enemy or the survivor
- **THEN** it does not hit them and continues its arc

#### Scenario: Projectile lands on world geometry

- **WHEN** an acid projectile contacts world geometry or its lifetime expires
- **THEN** it despawns and creates one hazard pool at that horizontal position

### Requirement: Detonation behavior

An archetype with a `Detonate` behavior SHALL arm an irreversible fuse when the survivor enters its
trigger radius while it is engaged, and MUST detonate when the fuse elapses or when it dies while
armed. A detonation MUST damage the survivor and every non-`Dying` enemy inside the authored radius,
MUST use explosive damage, MUST apply horizontal knockback away from the blast, and MUST resolve at most
once per entity.

#### Scenario: Proximity arms the fuse

- **WHEN** an engaged detonating enemy has the survivor inside its trigger radius
- **THEN** it arms its fuse and roots in place for the authored fuse duration

#### Scenario: The fuse is irreversible

- **WHEN** an armed enemy's survivor leaves the trigger radius
- **THEN** the fuse continues and the enemy still detonates when it elapses

#### Scenario: Blast hits the survivor and nearby enemies

- **WHEN** a detonation resolves
- **THEN** the survivor and every non-`Dying` enemy inside the radius receive explosive damage and
  outward knockback

#### Scenario: Damage falls off with distance

- **WHEN** a target is farther from the blast center
- **THEN** it receives less damage than a target closer to the center, down to zero at the radius

#### Scenario: Killing an armed enemy still detonates it

- **WHEN** an armed enemy is killed before its fuse elapses
- **THEN** it detonates once at its death position and the kill pipeline still reports one kill

#### Scenario: Dying enemies are not damaged by blasts

- **WHEN** a blast resolves near an enemy that is already `Dying`
- **THEN** that enemy receives no damage and no second kill message is emitted

### Requirement: Spitter archetype

The Spitter SHALL spawn with moderate health and speed and a ranged acid attack that leaves a hazard
pool. Its authored `scrap_value` MUST be awarded through the death pipeline, its pool MUST outlive it,
and it MUST respect its spawn policy.

#### Scenario: Spitter leaves acid behind

- **WHEN** a Spitter's projectile lands
- **THEN** a hazard pool exists at the impact point and damages the survivor over time

#### Scenario: Pool outlives the Spitter

- **WHEN** a Spitter dies after its projectile landed
- **THEN** the existing pool continues until its authored lifetime expires

#### Scenario: Spitter awards authored scrap

- **WHEN** a Spitter dies
- **THEN** the kill message carries the Spitter's authored `scrap_value`

#### Scenario: Spitter respects its spawn policy

- **WHEN** elapsed run time is below the Spitter's `unlock_secs`
- **THEN** the Spitter is excluded from the spawn roll

### Requirement: Exploder archetype

The Exploder SHALL spawn with low health and moderate speed, MUST close to proximity range, and MUST
detonate on proximity or death. Its blast MUST damage other enemies as well as the survivor, and its
authored `scrap_value` MUST be awarded through the death pipeline.

#### Scenario: Exploder threatens a crowd

- **WHEN** the survivor enters an Exploder's trigger radius
- **THEN** the Exploder arms and detonates after its fuse, damaging everything nearby

#### Scenario: Exploder damages its own horde

- **WHEN** an Exploder detonates inside a group of other enemies
- **THEN** those enemies receive explosive damage

#### Scenario: Exploder killed at range still blasts

- **WHEN** an Exploder is killed before reaching the survivor
- **THEN** it detonates at its death position

#### Scenario: Exploder awards authored scrap

- **WHEN** an Exploder dies from any cause
- **THEN** its kill message reports the archetype's authored `scrap_value` exactly once

### Requirement: Archetype piercing cost

Every archetype SHALL author a positive `pierce_cost` that defines both the shot budget it consumes and
the multiplier applied to a weapon's piercing penalty. Heavy archetypes MUST cost more than light ones; the
Tank MUST carry the highest cost in the authored roster.

#### Scenario: Cost consumes shot budget

- **WHEN** a projectile crosses an archetype
- **THEN** its remaining piercing power is reduced by that archetype's authored `pierce_cost`

#### Scenario: Cost multiplies damage drain

- **WHEN** a projectile crosses an archetype with a non-zero weapon penalty
- **THEN** its current damage is reduced by `pierce_cost * penalty`

#### Scenario: Tank costs the most

- **WHEN** the authored piercing costs are compared
- **THEN** the Tank's cost is greater than every other archetype's cost, and a Runner costs less than a
  Walker
