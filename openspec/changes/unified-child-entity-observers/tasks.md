## 1. Shared Block Visual Link & Turret Barrel Observer

- [ ] 1.1 Add `BlockVisual(pub Entity)` component in `core/src/content/block/placement.rs` (or `component.rs`) and insert it on the block entity in `spawn_block_entity`.
- [ ] 1.2 Add `TurretBarrelSpec { image, offset, pixel_size, initial_rotation }` component in `core/src/content/block/feature/turret/component.rs` and update `TurretSpec::install` (formerly `TurretComponent::insert`) to insert `TurretBarrelSpec` instead of spawning the barrel imperatively.
- [ ] 1.3 Add `spawn_turret_barrel` observer (`On<Add, TurretBarrelSpec>`) in `core/src/content/block/feature/turret/system.rs` and register it in `TurretPlugin`, guarding against duplicate spawning if `TurretBarrel` already exists.
- [ ] 1.4 Update `BlockSnapshot::apply` (`core/src/save/world/snapshot/block.rs`) to restore saved `barrel_rotation` cleanly alongside the `On<Add, TurretBarrelSpec>` observer.

## 2. Shield Barrier & Disabled Observers

- [ ] 2.1 Replace `spawn_shield_barriers` (`Added<Shield>` system in `core/src/content/block/feature/shield/system.rs`) with an `On<Add, Shield>` observer (`spawn_shield_barrier`) taking optional render asset resources (`Option<ResMut<Assets<Mesh>>>`, `Option<ResMut<Assets<ShieldMaterial>>>`) and skipping when `Disabled` or `shield.barrier.is_some()`.
- [ ] 2.2 Add `disable_shield_on_disabled` observer (`On<Add, Disabled>`) in `shield/system.rs` that immediately sets barrier `CollisionGroups` to `Group::NONE` and `Visibility` to `Visibility::Hidden`.
- [ ] 2.3 Register both observers in `ShieldPlugin` (`shield/mod.rs`) and gate `update_shield_state` / `update_shield_visuals` appropriately.

## 3. Overdrive Field Observer & Editor Reuse

- [ ] 3.1 Add `OverdriveField(pub Entity)` link component in `core/src/content/block/feature/power/field/component.rs` and extract `build_overdrive_material(radius, assets, materials)` in `field/lib.rs` (removing the redundant `Transform` double-insert).
- [ ] 3.2 Replace `spawn_field_on_booster` (`Added<Booster>` system) with an `On<Add, Booster>` observer (`spawn_overdrive_field`) guarded by `Without<OverdriveField>` and `Without<Disabled>`, and scope `sync_field_visibility` into `SimulationSet::Present` in `FieldPlugin`.
- [ ] 3.3 Update Ship Editor placed blocks (`scene/editor/ship_list.rs`) to attach `BlockVisual`, `TurretBarrelSpec`, and `Booster` so observers spawn barrels and fields identically; update `update_placement_preview` (`scene/editor/grid.rs`) to use `build_overdrive_material`.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests proving `TurretBarrel`, `Shield.barrier`, and `OverdriveField` spawn on the first `world.flush()` (including chunk-restore path), do not duplicate on re-insert, and deactivate on debris `Disabled`.
- [ ] 4.2 Add unit test verifying editor overdrive field material uses the booster's actual radius.
- [ ] 4.3 Update `docs/architecture.md` and `docs/scenes.md` (spec only, no notes) to document the observer-driven child entity contract.
