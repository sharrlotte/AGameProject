## ADDED Requirements

### Requirement: Authored enemy archetype roster

The simulation SHALL define every enemy archetype in `game/assets/config/enemies/enemies.ron` as a
uniquely keyed entry in `EnemiesConfig::archetypes`. Each entry MUST carry the archetype identity, combat
stats, `scrap_value`, a spawn policy, and an ordered behavior list. `EnemiesConfig` MUST validate the
roster before it is used and MUST reject duplicate kinds, a missing Walker, non-positive or non-finite
stats, invalid behavior values, and a roster with no positive spawn weight.

#### Scenario: Authored roster loads

- **WHEN** `enemies.ron` is parsed
- **THEN** the roster contains unique entries for Walker, Runner, and Tank with positive stats and at
  least one positive spawn weight

#### Scenario: Duplicate archetype is rejected

- **WHEN** validation runs on a roster containing two entries with the same `EnemyKind`
- **THEN** validation fails with an error naming the duplicate kind

#### Scenario: Walker is required

- **WHEN** validation runs on a roster with no Walker entry
- **THEN** validation fails because Walker is the guaranteed spawn fallback

#### Scenario: Invalid behavior parameters are rejected

- **WHEN** an archetype declares an Armor behavior with `arc_deg` outside `(0, 360]` or a negative
  multiplier, or a Melee behavior with a negative damage, cooldown, wind-up, reach, or knockback value
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
steer toward its target or start a melee wind-up. The Tank's mass-based knockback resistance MUST make
ordinary bullet hits insufficient to stagger it.

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
`Alert`, and MUST NOT steer while in `Idle`, `Stagger`, or an active melee wind-up. Steering MUST include
a separation term that pushes overlapping enemies apart without requiring a spatial grid.

#### Scenario: Engage chases the target

- **WHEN** an enemy is in `Engage` with a live target
- **THEN** its horizontal velocity points toward the target

#### Scenario: Wind-up roots the attacker

- **WHEN** an enemy is in an active melee wind-up
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

Editing `enemies.ron` while the game is running SHALL update live enemy stats and behavior values and
SHALL apply spawn policy and scrap changes to subsequent spawns and kills. A structural change that adds
or removes a behavior on an existing archetype MUST NOT panic; it SHALL log a warning and take effect on
the next deploy.

#### Scenario: Tank armor hot reloads

- **WHEN** `front_mult` changes in `enemies.ron` during a run
- **THEN** live Tanks use the new multiplier for subsequent hits

#### Scenario: Spawn policy hot reloads

- **WHEN** `unlock_secs`, `weight`, or `max_active` changes during a run
- **THEN** subsequent spawn rolls use the new policy

#### Scenario: Structural behavior change is safe

- **WHEN** a behavior is added to or removed from an archetype during a run
- **THEN** the game logs a warning and does not panic
