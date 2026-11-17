## ADDED Requirements

### Requirement: Turrets target the enemy core

Every ship-targeting strategy SHALL aim at the target ship's core position; if the core is absent the target SHALL be invalid. No strategy SHALL use an arbitrary first block.

#### Scenario: Aim point is the core
- **WHEN** a target ship has many blocks
- **THEN** the turret aim point equals the target's core center

### Requirement: Closest selects the nearest enemy core

The `Closest` strategy SHALL select the nearest enemy core within range, tie-breaking by entity id.

#### Scenario: Deterministic tie-break
- **WHEN** two enemies are equidistant
- **THEN** the lower entity id is selected

### Requirement: MostHP selects the highest current ship HP

The `MostHP` strategy SHALL select the enemy within range with the highest current `ShipStats.health`, tie-breaking by distance and then entity id.

#### Scenario: Current HP wins
- **WHEN** a damaged high-max-HP ship and a healthy low-HP ship are in range
- **THEN** the healthy ship is selected

### Requirement: Point defense intercepts bullets and missiles only

The `EnemyBullet` strategy SHALL query hostile `Interceptable` projectiles, SHALL consider bullets and missiles, SHALL NOT consider lasers or beams, and SHALL choose the nearest target with missiles prioritized when close. It SHALL aim with lead prediction and despawn a hit projectile; a destroyed missile SHALL NOT trigger its blast.

#### Scenario: Lasers are not interceptable
- **WHEN** a hostile bullet and a laser are in range
- **THEN** only the bullet can be targeted

#### Scenario: Missile is destroyed safely
- **WHEN** a missile is intercepted
- **THEN** it disappears without dealing blast damage

#### Scenario: Lead prediction
- **WHEN** a moving bullet is targeted
- **THEN** the aim point leads its velocity using the turret projectile speed

### Requirement: Turret state and fire gate are explicit

`TurretState` SHALL contain `target: Option<TurretTarget>`, `aim_point`, and `can_fire`; the fire gate SHALL compare barrel forward with `aim_point`, and `TurretTarget` SHALL distinguish ship and projectile targets. Targets SHALL be skipped on save and reacquired after load.

#### Scenario: Barrel not aligned
- **WHEN** a turret's barrel has not turned to the aim point
- **THEN** it does not fire

### Requirement: Continuous laser and pulling-beam visuals match their hitboxes

The continuous laser and pulling-beam visual entities SHALL be children of the rotating barrel entity, SHALL use local transform `(0.0, range / 2.0, LAYER_BULLET)`, and SHALL extend along barrel-local `+Y`, matching the hitbox direction. The projectile laser sprite SHALL be centered at `(0.0, length / 2.0)`, not shifted laterally.

#### Scenario: Rotated barrel alignment
- **WHEN** the barrel is rotated 90 degrees
- **THEN** the beam visual and the hitbox both point along the barrel forward direction

#### Scenario: Projectile laser is axial
- **WHEN** a projectile laser spawns
- **THEN** its sprite is centered on its travel axis

### Requirement: Combat edge cases are handled

Empty turret barrels SHALL be rejected at config validation and guarded at runtime without division by zero. A pulling beam SHALL apply pull once per target ship per tick. Damage text SHALL use `LAYER_DAMAGE_TEXT = 400.0`. Missile visual and hitbox z-order SHALL be aligned. Fire and cooldown timers SHALL reset on spawn and pause correctly. Transient combat visuals SHALL be scoped to `SceneState::Game`.

#### Scenario: Empty barrels are safe
- **WHEN** a turret has no barrels
- **THEN** it does not panic and does not fire

#### Scenario: Pull once per ship
- **WHEN** a pulling beam overlaps several blocks of one ship
- **THEN** that ship receives one pull per tick

#### Scenario: Damage text is in camera range
- **WHEN** damage text spawns
- **THEN** its z is inside the 2D camera range
