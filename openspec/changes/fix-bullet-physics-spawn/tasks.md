## 1. Bullet Component & Plugin Cleanup

- [x] 1.1 Remove `velocity: Vec2` from `Bullet` in `core/src/content/bullet/component.rs` (check if any tests or callers reference `Bullet::velocity`).
- [x] 1.2 Remove `handle_bullets` from `BulletPlugin` in `core/src/content/bullet/mod.rs` and delete `handle_bullets` from `core/src/content/bullet/system.rs`.

## 2. Projectile Spawn GlobalTransform & RigidBody

- [x] 2.1 In `spawn_bullet` (`core/src/content/bullet/system.rs`), construct `transform` and insert `GlobalTransform::from(transform)` for both `ShootingProjectile::Bullet` and `ShootingProjectile::Missile`.
- [x] 2.2 In `spawn_bullet` (`ShootingProjectile::Bullet`), add `RigidBody::KinematicVelocityBased` and `Velocity::linear(dir * *speed)` alongside `Ccd::enabled()` and `Sensor`.
- [x] 2.3 In `handle_missile_collision` (`core/src/content/bullet/system.rs`), construct `let transform = Transform::from(*transform);` and insert `GlobalTransform::from(transform)` on the blast AoE sensor entity.
- [x] 2.4 Update `destroy_out_of_range_bullet` (`core/src/content/bullet/system.rs`) to query `(Entity, &Bullet, &Transform)` and use `transform.translation.truncate()` instead of `GlobalTransform`.

## 3. Documentation & Verification

- [x] 3.1 Update `docs/combat.md` projectile table and notes to reflect `RigidBody::KinematicVelocityBased`, `Velocity`, and `GlobalTransform::from(transform)` at spawn.
- [x] 3.2 Run `cargo test` (or targeted bullet/combat tests) and verify `cargo dev` via BRP in Quick Battle that enemy `colaris` bullets visibly travel from enemy ships, collide with the player normally, and do not hit `(0, 0)` on frame 1.
