## Why

Feature child entities are currently spawned through three inconsistent mechanisms: turret barrels are spawned imperatively inside `TurretComponent::insert` (`turret/component.rs:86-99`), shield barriers are spawned lazily in `FixedUpdate` via `Added<Shield>` (`shield/system.rs:13-83`), overdrive fields are spawned in an unscoped `Update` system via `Added<Booster>` with a broken `Without<Field>` parent guard (`power/field/system.rs:37-72`), and the Ship Editor duplicates barrel and overdrive-field spawning manually (using the wrong 256px ring material for 1028px boosters). Because `stream_chunks` restores ships in `FixedUpdate::Cleanup` after `Weapons`, shields also suffer a 1-tick gap without a barrier collider on restore.

## What Changes

- Standardize all feature-owned child entities (turret barrels, shield barriers, overdrive fields) onto Bevy 0.20 `On<Add, T>` observers so feature insertion (`Feature::install`) inserts pure data components only.
- Add data components describing visual child attachment (`TurretBarrelSpec` / `VisualChild(Entity)` on block, `OverdriveField(Entity)` link on booster) so observers run synchronously during command flush and prevent duplicate child spawning cleanly (fixing the broken `Without<Field>` guard on boosters and eliminating the 1-tick shield restore gap).
- Add an `On<Add, Disabled>` observer for `Shield` (matching turret, booster, radar, and beams) to disable shield barriers immediately when a block becomes debris or shed.
- Share the overdrive field material/sizing builder and observer-driven child spawning with the Ship Editor so placed blocks and previews stop duplicating barrel/field spawning and render accurate `ring_ratio` for every booster radius.
- Remove `spawn_shield_barriers` and `spawn_field_on_booster` polling systems and verify restore/chunk-spawn ordering (including barrel rotation application and debris `sync_debris_disabled` interaction).

## Capabilities

### New Capabilities
- `feature-child-observers`: Standardized Bevy 0.20 `On<Add, T>` observer lifecycle for all feature child entities (turret barrels, shield barriers, overdrive fields) with immediate command-flush spawning and editor reuse.

### Modified Capabilities
- `collapsed-features`: Update child-entity exception specification to cover `Visual`, turret barrel, shield `Barrier`, and overdrive `Field` child entities spawned uniformly via `On<Add, T>` observers.
- `shield-defense`: Specify `Shield` (replacing stale `ShieldEmitter` name), observer-based barrier spawning on `On<Add, Shield>`, and `On<Add, Disabled>` immediate barrier deactivation.
- `overdrive`: Specify observer-based `Field` spawning on `On<Add, Booster>` with proper per-radius `OverdriveMaterial` sizing in both battle and Ship Editor.

## Non-Goals

- Changing transient runtime VFX or projectile child entities (`FlashOverlay`, active `LaserBeam`/`PullingBeam`, `NoAmmoWarningUI`).
- Persisting `Shield` state or `Disabled` markers (covered by `colocated-component-snapshots`).

## Impact

- `core/src/content/block/feature/turret/` (`component.rs`, `mod.rs`, `system.rs`)
- `core/src/content/block/feature/shield/` (`component.rs`, `mod.rs`, `system.rs`)
- `core/src/content/block/feature/power/field/` (`component.rs`, `lib.rs`, `mod.rs`, `system.rs`)
- `core/src/content/block/component.rs` (`spawn_block_entity`)
- `core/src/scene/editor/` (`ship_list.rs`, `grid.rs`)
- `core/src/save/world/snapshot/block.rs` (barrel rotation restoration)
- `docs/architecture.md` and `docs/scenes.md` (spec-only updates)
