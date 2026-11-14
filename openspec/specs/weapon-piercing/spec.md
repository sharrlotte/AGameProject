# weapon-piercing

Multi-body projectile penetration: authored power and penalty, damage drain driven by enemy pierce cost, nearest-first deterministic collision resolution, hit-once bookkeeping, knockback decay, and termination safety bounds.

## Requirements

### Requirement: Weapon piercing profile

The simulation SHALL expose a piercing profile on `WeaponDefinition` containing `power` (remaining body
budget) and `penalty` (damage lost per unit of enemy piercing cost). Both fields MUST default to zero when
omitted, MUST be finite and non-negative, and a weapon whose `power` exceeds `MAX_HITS * MIN_PIERCE_COST`
MUST be rejected during roster validation.

#### Scenario: Omitted profile means no piercing

- **WHEN** a weapon is authored without a piercing profile
- **THEN** both `power` and `penalty` are zero and the weapon keeps its single-target behaviour

#### Scenario: Negative piercing values are rejected

- **WHEN** validation runs on a weapon with a negative or non-finite `power` or `penalty`
- **THEN** validation fails with an error naming the weapon

#### Scenario: Excess piercing power is rejected

- **WHEN** validation runs on a weapon whose `power` could exceed `MAX_HITS` bodies at the smallest legal
  enemy cost
- **THEN** validation fails instead of allowing a shot that would overflow the hit log

### Requirement: Enemy cost drives damage drain and termination

Each body crossed SHALL consume that archetype's `pierce_cost` from the shot's remaining power and SHALL
subtract `pierce_cost * penalty` from the shot's current damage. The shot MUST deal its current damage
before either drain, MUST continue only while both remaining power and current damage are greater than
zero, and MUST stop after the body that exhausts either. World obstacles MUST stop the shot regardless of
remaining power. Damage MUST never go below zero.

#### Scenario: Damage decays by archetype cost

- **WHEN** a shot with non-zero power and penalty crosses two bodies with the same cost
- **THEN** the first body takes the full current damage and the second takes damage reduced by
  `cost * penalty`

#### Scenario: A Tank exhausts a rifle shot

- **WHEN** a rifle shot reaches a Tank whose `pierce_cost` is greater than or equal to the remaining power
- **THEN** the Tank takes the current damage and the shot stops after it

#### Scenario: A Runner costs less than a Walker

- **WHEN** the same shot crosses a Runner and then a Walker
- **THEN** the Runner consumes less power and less damage than the Walker would

#### Scenario: Zero power preserves one-hit behaviour

- **WHEN** a weapon has zero piercing power
- **THEN** its projectile deals damage to the first enemy and despawns, exactly as before this change

#### Scenario: A world obstacle always stops the shot

- **WHEN** a projectile contacts world geometry
- **THEN** it despawns after applying any nearer enemy hits from the same frame, regardless of remaining
  power

#### Scenario: Armor applies after the drain

- **WHEN** a drained hit reaches an armored archetype
- **THEN** the archetype's armor multiplier is applied to the drained damage

#### Scenario: Damage never goes negative

- **WHEN** `cost * penalty` exceeds the shot's current damage
- **THEN** the body still takes the pre-drain damage and the shot stops with zero remaining damage

### Requirement: Nearest-first multi-body resolution

The simulation SHALL resolve same-frame projectile collisions in increasing projection distance along the
projectile travel direction, using a deterministic tiebreak, and MUST exclude enemies marked `Dying`.
Obstacles and enemies MUST share the same ordering.

#### Scenario: Two bodies in one frame resolve in travel order

- **WHEN** a projectile starts contact with two enemies in the same frame
- **THEN** the enemy nearer along the travel direction takes damage first and consumes budget first

#### Scenario: A Tank behind a Walker does not eat the budget first

- **WHEN** a Walker and a Tank register in the same frame, with the Walker nearer
- **THEN** the Walker is resolved before the Tank even if the collision events arrive in the opposite order

#### Scenario: A nearer obstacle stops the shot before a farther enemy

- **WHEN** a projectile registers an obstacle and an enemy in the same frame, with the obstacle nearer
- **THEN** the shot stops at the obstacle and the enemy takes no damage

#### Scenario: Dying enemies are skipped

- **WHEN** a projectile contacts an enemy already marked `Dying`
- **THEN** that enemy receives no piercing hit and the shot continues or stops by its own state

### Requirement: Hit-once guarantee

A projectile MUST NOT damage the same enemy more than once during its flight. It SHALL record hit entities
in a bounded log, and when the log is full the shot MUST stop rather than risk a duplicate hit.

#### Scenario: Leaving and re-entering a body does not double-hit

- **WHEN** an enemy leaves and re-enters a projectile's sensor during the same flight
- **THEN** the enemy receives damage only once and the repeated contact is ignored

#### Scenario: A full hit log stops the shot

- **WHEN** a projectile has recorded the maximum number of distinct hit entities
- **THEN** it stops after the current body instead of continuing untracked

### Requirement: Knockback decays with retained damage

Each pierced body's knockback SHALL scale by the ratio of the shot's current pre-armor damage to its
initial pre-armor damage, so later bodies receive less push.

#### Scenario: The first body takes full knockback

- **WHEN** a pierce shot hits its first enemy
- **THEN** the knockback equals the weapon's authored knockback

#### Scenario: Later bodies take reduced knockback

- **WHEN** a shot has lost damage to earlier bodies
- **THEN** the next body's knockback is scaled below the weapon's authored value

### Requirement: Piercing preserves the damage pipeline

Piercing SHALL change only the `amount` and `knockback` carried by `DamageMessage`. Armor resolution,
stagger, aggro, death marking, and kill accounting MUST continue to run exactly once per damaged enemy
through the existing damage pipeline.

#### Scenario: Kill accounting stays single-sourced

- **WHEN** a piercing shot kills two enemies in one frame
- **THEN** two `EnemyKilledMessage` messages are emitted through the existing death pipeline, one per
  enemy, with each enemy's authored scrap value

#### Scenario: Stagger still reads the reduced knockback

- **WHEN** a later body receives reduced knockback below the stagger threshold
- **THEN** it is not staggered, while an earlier body above the threshold still is
