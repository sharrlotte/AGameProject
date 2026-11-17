## 1. Ship Identity & Weapon Relative Timers

- [ ] 1.1 Separate `Ship.id` (unique ship instance `Uuid`) from `Ship.core_id` (core block `BlockPlacement.id`) in `core/src/content/ship/component.rs` and `core/src/content/block/component.rs`, add `ShipDesign::spawn_with_id`, update `has_core` in `core/src/content/block/feature/turret/system.rs` to check `ship.core_id`, and preserve `ship.id` in `apply_ship_mutations` (`core/src/content/ship/mutation.rs`).
- [ ] 1.2 Replace absolute `elapsed_secs` timestamps in `SingleShootPatternState`, `BurstShootPatternShootingState`, and `LaserBeamPatternShootingState` (`core/src/content/block/component.rs` and `core/src/content/bullet/system.rs`) with relative remaining/elapsed seconds (`cooldown_remaining`, `barrel_delay_remaining`, `elapsed`) ticked by `time.delta_secs()`.

## 2. Modular Save Architecture (`design/`, `campaign/`, `battle/`, `document.rs`)

- [ ] 2.1 Create `core/src/save/design/` (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`) with `DesignSavePlugin` and `ShipCatalog` as a Bevy `Resource` seeded once at startup and updated when saving designs, and update `core/src/system.rs`, `core/src/mission/`, and `core/src/scene/editor/` to consume the `ShipCatalog` resource.
- [ ] 2.2 Create `core/src/save/campaign/` (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`) with `CampaignSavePlugin`, `CampaignSave` as an in-memory cached `Resource`, and campaign progression / reset handlers.
- [ ] 2.3 Create `core/src/save/battle/` (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`) with `BattleSavePlugin`, `BattleSaveState` (`Resource`), and `WorldSave` v4 schema (`SAVE_SCHEMA_VERSION = 4`) including ship cargo (`CargoSave`), battery/turret charges, turret barrel rotation, `AiRuntimeSave` (`AiBehaviorSave` with target ship `Uuid`s, `spawn_position`, `rule_cooldowns`, `timer_secs`), `CameraSave` (`following: Option<Uuid>`), and `ControlSave` (`controlling: Option<Uuid>`, `pilot_mode: bool`).
- [ ] 2.4 Implement direct hierarchical `WorldSave::capture` → `ShipSnapshot::capture` → `BlockSnapshot::capture(world, grid_block)` with $O(1)$ `GridBlock` lookup and two-pass `WorldSave::restore` (pass 1: spawn ships with saved `id`, blocks, cargo, power/weapon charges, barrel rotation, and visible item drops with rarity `Sprite`; pass 2: resolve `Uuid -> Entity` for AI behavior targets, `ControlledShip`, `CameraTarget`, `ControlMode`, and restore `BattlePhase::Ended` when an outcome is present).
- [ ] 2.5 Wire `core/src/save/mod.rs` (`SavePlugin`) and update `core/src/scene/home.rs`, `core/src/scene/pause.rs`, `core/src/scene/setting.rs`, and `core/src/mission/mod.rs` to use the new `save::design`, `save::campaign`, and `save::battle` modules and resources, removing old flat files (`catalog.rs`, `io.rs`, `schema.rs`, `snapshot.rs`).

## 3. Verification & Spec Documentation

- [ ] 3.1 Add and update unit/integration tests across `core/src/save/` covering `WorldSave` v4 round-trip (two ships from the same design, cargo, battery/turret charge, barrel rotation, AI runtime targets/cooldowns, pilot/camera targets, relative weapon timers, drop sprites, and battle outcome phase), version mismatch (rejecting v3), and campaign/design resource caching.
- [ ] 3.2 Update `/docs/save.md` specification to document the `SAVE_SCHEMA_VERSION = 4` schema, modular `save/` layout, and resource-backed persistence flow.
