## ADDED Requirements

### Requirement: Fixed-schedule projectile spawns initialize GlobalTransform
Every root projectile and blast-sensor entity spawned during `FixedUpdate` (`Bullet`, `Missile`, and missile blast AoE sensors) SHALL initialize `GlobalTransform::from(transform)` alongside `Transform` so Rapier's fixed-schedule physics backend immediately reads the entity's true world position rather than `GlobalTransform::IDENTITY` (`(0, 0, 0)`).

#### Scenario: Enemy bullet spawned far from origin does not collide at origin
- **WHEN** an enemy turret at `(5708, 31)` spawns a `Bullet` while the player ship is at `(0, 0)`
- **THEN** the bullet's `GlobalTransform` equals `(5708, 31)` on the first Rapier physics step, does not collide with the player at `(0, 0)`, and travels visibly toward its target

#### Scenario: Missile blast sensor spawns at impact position
- **WHEN** a `Missile` collides and spawns its blast AoE `Collider` at the impact `GlobalTransform`
- **THEN** the blast sensor entity is spawned with both `Transform::from(*transform)` and `GlobalTransform::from(Transform::from(*transform))`

### Requirement: Bullet entities move via KinematicVelocityBased RigidBody
`Bullet` entities SHALL be spawned with `RigidBody::KinematicVelocityBased` and `Velocity::linear(dir * speed)` (with `Ccd::enabled()` and `Sensor`), and SHALL NOT be moved by a manual `Transform` integration system.

#### Scenario: Bullet travels at configured speed and collides with enemy blocks
- **WHEN** a turret fires a `Bullet` with speed `S` along direction `dir`
- **THEN** Rapier integrates the bullet's position at `dir * S` per second, updates `Transform`, and emits `CollisionEvent::Started` on contact with enemy block hitboxes

#### Scenario: Point-defense turrets can read Bullet velocity
- **WHEN** a `TargetingStrategy::EnemyBullet` turret queries `With<Interceptable>` and `&Velocity`
- **THEN** enemy `Bullet` entities are matched by the query with their current linear velocity

### Requirement: Bullet range culling reads Transform directly
`destroy_out_of_range_bullet` SHALL compare `bullet.shooted_at` against the bullet's `&Transform` translation rather than `&GlobalTransform`.

#### Scenario: Multi-step FixedUpdate does not prematurely despawn distant bullets
- **WHEN** `FixedUpdate` runs multiple ticks before `PostUpdate` propagates transforms
- **THEN** a newly spawned `Bullet` beyond `bullet.range` from `(0, 0)` is evaluated against its own `Transform.translation` and is not despawned until its traveled distance exceeds `bullet.range`
