## Why

Enemy bullets appear invisible while still dealing damage at `(0, 0)` because projectiles spawned in `FixedUpdate` (`SimulationSet::Weapons`) only set `Transform` and omit `GlobalTransform`. Since `RapierPhysicsPlugin` runs `.in_fixed_schedule()` before `PostUpdate` propagates `Transform` to `GlobalTransform`, Rapier places the newly spawned bullet collider at `GlobalTransform::IDENTITY` (`(0, 0, 0)`), colliding immediately with the player at the origin and despawning on frame 1 while its visual sprite sits offscreen at the enemy's position. Additionally, `Bullet` currently lacks a `RigidBody` and moves via manual `Transform` updates, which disables Rapier's swept CCD (`Ccd::enabled()`), and `destroy_out_of_range_bullet` reads a potentially stale `GlobalTransform`.

## What Changes

- Initialize `GlobalTransform::from(transform)` explicitly when spawning `Bullet`, `Missile`, and missile blast AoE sensors in `FixedUpdate` so Rapier immediately reads the true world position.
- Add `RigidBody::KinematicVelocityBased` and `Velocity::linear(dir * *speed)` to spawned `Bullet` entities so Rapier integrates bullet motion and swept CCD natively.
- Remove the manual `handle_bullets` system and `Bullet::velocity` field (replaced by Rapier `Velocity`).
- Update `destroy_out_of_range_bullet` to read `&Transform` instead of `&GlobalTransform` (bullets are root entities with no parent).
- Update `/docs/combat.md` to reflect the new `Bullet` physics components and movement model.

## Capabilities

### New Capabilities
- `bullet-physics-spawn`: Synchronized `GlobalTransform` initialization for `FixedUpdate`-spawned projectile colliders, `KinematicVelocityBased` rigid-body movement with swept CCD for `Bullet` entities, and `Transform`-based range culling.

### Modified Capabilities
<!-- None: existing specs do not define bullet spawn GlobalTransform or RigidBody mechanics. -->

## Impact

- `core/src/content/bullet/component.rs`: Remove `velocity` from `Bullet` (or keep only if needed elsewhere; check callers).
- `core/src/content/bullet/system.rs`: Update `spawn_bullet`, `handle_missile_collision`, `destroy_out_of_range_bullet`; remove `handle_bullets`.
- `core/src/content/bullet/mod.rs`: Remove `handle_bullets` from `BulletPlugin`.
- `docs/combat.md`: Update projectile table to document `RigidBody::KinematicVelocityBased`, `Velocity`, and `GlobalTransform` initialization.
