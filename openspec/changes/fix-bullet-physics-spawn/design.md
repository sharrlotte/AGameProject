## Context

The game runs its simulation (`Ai`, `Control`, `Weapons`, `Damage`, `Death`, `Cleanup`) and `RapierPhysicsPlugin::in_fixed_schedule()` inside `FixedUpdate` / `FixedPostUpdate`. Bevy's `TransformSystems::Propagate` (`Transform` → `GlobalTransform`) only runs in `PostUpdate`, after `FixedMain` completes.

Currently:
1. `spawn_bullet` (`core/src/content/bullet/system.rs`) spawns `Bullet` and `Missile` with `Transform` but without `GlobalTransform`, leaving `GlobalTransform` at `IDENTITY` (`(0, 0, 0)`) during the fixed-schedule Rapier step. Enemy bullets spawned at `(5708, 31)` have their collider placed at `(0, 0)` on frame 1, colliding with the player's core and despawning before ever rendering in flight.
2. `handle_missile_collision` similarly spawns a blast AoE `Collider` from `Transform::from(*transform)` without `GlobalTransform`.
3. `Bullet` has `Ccd::enabled()` without a `RigidBody` and is moved manually via `handle_bullets` (`transform.translation += bullet.velocity * dt`), so Rapier does not perform swept CCD and `EnemyBullet` point-defense queries (`With<Interceptable>`, `&Velocity`) cannot match `Bullet`s.
4. `destroy_out_of_range_bullet` queries `&GlobalTransform` instead of `&Transform`, which can read `(0, 0)` on multi-step `FixedUpdate` frames.

## Goals / Non-Goals

**Goals:**
- Ensure every `Bullet`, `Missile`, and blast AoE collider spawned in `FixedUpdate` has an accurate `GlobalTransform` before Rapier reads it, with zero extra `FixedPostUpdate` transform-propagation passes.
- Move `Bullet` entities natively via Rapier `RigidBody::KinematicVelocityBased` + `Velocity` so `Ccd::enabled()` works and point-defense `Velocity` queries match `Bullet`s.
- Make `destroy_out_of_range_bullet` immune to `GlobalTransform` propagation lag by reading `&Transform`.

**Non-Goals:**
- Adding `sync_simple_transforms` / `propagate_parent_transforms` to `FixedPostUpdate`.
- Changing `Missile` movement (`handle_missiles` + `MissileHitbox` child) or `Laser` raycasting (`handle_laser`) beyond `GlobalTransform::from` at spawn.
- Fixing `handle_laser`'s double `LAYER_BULLET` Z-offset (out of scope).

## Decisions

1. **Explicit `GlobalTransform::from(transform)` at spawn instead of `FixedPostUpdate` transform propagation**
   - *Rationale:* Avoids adding an extra schedule pass in `FixedPostUpdate`. Root projectiles (`Bullet`, `Missile`, blast sensor) have no parent, so `GlobalTransform::from(transform)` is exact.
   - *Alternative considered:* Running `sync_simple_transforms` + `propagate_parent_transforms` in `FixedPostUpdate` before `PhysicsSet::SyncBackend`. Rejected to avoid extra per-tick schedule overhead per user preference.

2. **`RigidBody::KinematicVelocityBased` + `Velocity::linear(dir * *speed)` on `Bullet` (removing `handle_bullets` and `Bullet::velocity`)**
   - *Rationale:* Rapier integrates movement directly in `StepSimulation` and writes back to `Transform`, keeping collider and visual positions in sync without manual `Transform` mutation, enabling `Ccd::enabled()`, and populating `&Velocity` for `TargetingStrategy::EnemyBullet`.
   - *Alternative considered:* `RigidBody::Dynamic` with `GravityScale(0.0)` and `LockedAxes`. Rejected because bullets are sensors with no physical collision response needed; `KinematicVelocityBased` is lighter and cannot be pushed by impulses.

3. **`&Transform` in `destroy_out_of_range_bullet`**
   - *Rationale:* `Bullet` is always a root entity (no `ChildOf`), so `Transform.translation` is its authoritative world position.

## Risks / Trade-offs

- **[Risk] `KinematicVelocityBased` + `Sensor` bullet vs. `Dynamic` ship's child `Sensor` block hitbox may require `ActiveCollisionTypes` check** → `RapierPhysicsPlugin` default `ActiveCollisionTypes` includes `DYNAMIC_KINEMATIC`, and block hitboxes are children of `Dynamic` ship bodies. Verify live in `cargo dev` via BRP that enemy and player bullets both travel visually and trigger `CollisionEvent::Started` + `DamageEvent`.
- **[Risk] Future `FixedUpdate`-spawned colliders could forget `GlobalTransform::from(transform)`** → Document the invariant in `docs/combat.md`.
