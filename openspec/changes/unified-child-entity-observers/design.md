## Context

Currently, spawning child entities for block features is split across incompatible patterns:
1. **Imperative inside `insert`**: `TurretComponent::insert` (`turret/component.rs:86-99`) directly spawns `TurretSprite` under `visual` and attaches `TurretBarrel(barrel)` to `base`.
2. **Deferred `Added<T>` systems**:
   - `spawn_shield_barriers` (`shield/system.rs:13-83`) runs in `FixedUpdate::Weapons` using `(Without<Disabled>, Added<Shield>)`. Because `stream_chunks` restores ships in `FixedUpdate::Cleanup`, newly restored ships exist for an entire fixed physics step without their shield barrier collider.
   - `spawn_field_on_booster` (`power/field/system.rs:37-72`) runs in an unscoped `Update` system with `(Added<Booster>, Without<Field>)`. Because `Field` is attached to the *child* entity rather than the booster parent, `Without<Field>` is a no-op, and debris boosters still spawn `Field` children.
3. **Editor duplication**: `scene/editor/ship_list.rs:180-208` and `scene/editor/grid.rs:157-182` manually spawn turret barrel sprites and overdrive field meshes (reusing the hardcoded 256px `FieldAssets.material` even for `radius = 1028.0` boosters, distorting the dashed ring).

In Bevy 0.20, `On<Add, T>` observers fire **synchronously during command flush** (`ApplyDeferred` / `world.flush()`), not deferred to a later frame or schedule. This makes observers both reactive (working on spawn, runtime feature toggle, or restore) and order-safe for physics and snapshot restoration.

## Goals / Non-Goals

**Goals:**
- Make feature insertion insert **data components only** (zero `commands.spawn(...)` inside `Feature::install`).
- Spawn turret barrels (`On<Add, TurretBarrelSpec>`), shield barriers (`On<Add, Shield>`), and overdrive fields (`On<Add, Booster>`) through one consistent Bevy 0.20 observer pattern.
- Store explicit parent→child link components (`BlockVisual(Entity)`, `TurretBarrel(Entity)`, `Shield.barrier: Option<Entity>`, `OverdriveField(Entity)`) so observers are idempotent and cleanup/lookup is O(1).
- Ensure chunk/save restore works immediately within the same `world.flush()` without 1-tick gaps or barrel-rotation race conditions.
- Eliminate editor duplication and wrong-radius overdrive ring rendering.

**Non-Goals:**
- Changing runtime weapon beams (`LaserBeam`/`PullingBeam`) or hit-flash overlays (`FlashOverlay`).
- Adding shield visualization to the Ship Editor unless explicitly requested later.

## Decisions

### 1. `On<Add, T>` Observers Instead of `Added<T>` Polling Systems
- **Choice**: Register global observers via `app.add_observer(...)` in each owning feature plugin:
  - `TurretPlugin`: `spawn_turret_barrel` on `On<Add, TurretBarrelSpec>`.
  - `ShieldPlugin`: `spawn_shield_barrier` on `On<Add, Shield>` + `disable_shield_on_disabled` on `On<Add, Disabled>`.
  - `FieldPlugin`: `spawn_overdrive_field` on `On<Add, Booster>` (skips if `OverdriveField` already exists or `Disabled` is present; skips gracefully if `FieldAssets` is absent in headless tests).
- **Rationale**: Unlike `Added<T>` systems (which wait until their schedule runs — creating a 1-tick gap on `FixedUpdate::Cleanup` restores), Bevy 0.20 `On<Add, T>` observers execute synchronously when `Commands` flush. When `spawn_block_entity` inserts `BlockVisual(visual)` and `TurretBarrelSpec { image, offset, pixel_size, initial_rotation }`, `world.flush()` spawns the barrel and inserts `TurretBarrel(barrel)` before the next physics/tracking step runs.
- **Alternative Considered**: *Standardizing on `Added<T>` systems in `FixedPreUpdate`* — still leaves 1-tick gaps for entities spawned in `FixedUpdate::Cleanup` (chunk streaming) and breaks same-tick snapshot barrel rotation.

### 2. Pure Data Specs & Initial Barrel Rotation for Restore Ordering
- **Choice**:
  - `spawn_block_entity` inserts `BlockVisual(visual)` on `block`.
  - `TurretSpec::install` inserts `Turret`, `TurretState`, `Radar`, power/ammo/pattern components, and `TurretBarrelSpec { image, offset, pixel_size, initial_rotation: 0.0 }`.
  - During snapshot restore (`BlockSnapshot::apply`), setting `TurretBarrelSpec.initial_rotation` (or applying rotation via `On<Add, TurretBarrel>` / existing queued command after flush) ensures the barrel spawns directly at the restored rotation with zero timing fragility.
  - Because `sync_debris_disabled` (`On<Insert, BlockOwner>`) and `spawn_shield_barrier` (`On<Add, Shield>`) can fire in the same flush when restoring debris, `spawn_shield_barrier` checks `Option<&Disabled>` AND `disable_shield_on_disabled` (`On<Add, Disabled>`) hides/disables any already-spawned barrier (or despawns debris barrier) — guaranteeing identical final state regardless of observer trigger order!

### 3. Shared Visual/Observer Helpers for Ship Editor
- **Choice**:
  - Extract `build_overdrive_material(radius, assets, materials)` in `content/block/feature/power/field/lib.rs` so both `spawn_overdrive_field` and editor preview use identical `ring_ratio` calculation (fixing the 256px vs 1028px distortion).
  - Editor placed blocks attach `BlockVisual(visual)` plus `TurretBarrelSpec` and `Booster` (with an `EditorVisual` marker or `Field` visibility rule when `BlockOwner` is absent — which `sync_field_visibility` already treats as `is_selected = true` at `power/field/system.rs:90`!) so the exact same observers spawn barrels and overdrive fields in the Ship Editor without duplicating entity-construction code.
  - Editor placement preview (`grid.rs`) reuses the shared `build_overdrive_material` helper and `TurretBarrelSpec` sprite builder so tinting (`PreviewSprite`) stays clean without duplicating geometry math.

## Risks / Trade-offs

- **Headless unit tests without `Assets<Mesh>` / `Assets<ShieldMaterial>` / `FieldAssets`** → Observers take `Option<ResMut<Assets<Mesh>>>`, `Option<ResMut<Assets<ShieldMaterial>>>`, `Option<Res<FieldAssets>>` and early-return cleanly (with `debug!` log) when render assets are not initialized, keeping unit tests lightweight.
- **Observer ordering on debris spawn** → `spawn_shield_barrier` checking `Disabled` PLUS `disable_shield_on_disabled` on `On<Add, Disabled>` makes the outcome order-independent.
- **Duplicate child spawn on re-insertion** → Guard every observer with an explicit link check (`TurretBarrel`, `Shield.barrier.is_some()`, `OverdriveField`) so re-inserting a data component never leaks orphan children.
