## Why

The current save/load system (`core/src/save/`) suffers from structural coupling, repeated synchronous disk I/O during gameplay and UI frames, an $O(N \times M)$ entity scan per block during snapshot capture, and major gameplay state loss on load (ship cargo, battery and turret charge, turret barrel rotation, AI runtime targets/cooldowns, pilot/camera targets, item drop sprites, and battle outcome state are lost, while absolute weapon timestamps freeze firing across sessions). Rewriting `core/src/save/` into modular feature-grouped plugins with resource-backed caches and full v4 snapshot fidelity resolves these defects and aligns the persistence layer with `AGENTS.md` architecture rules.

## What Changes

- **Modular Save Architecture**: Restructure `core/src/save/` into feature-grouped submodules (`save/battle/`, `save/campaign/`, `save/design/`, and `save/document.rs`) following the `mod.rs`, `component.rs`, `system.rs`, `lib.rs` layout.
- **In-Memory Resource Caching**: Cache `ShipCatalog`, `CampaignSave`, and `BattleSaveState` as Bevy `Resource`s initialized at startup and mutated via events/messages rather than performing synchronous disk reads/writes inside UI and mission systems. Keep `GameSetting` in `assets/config/setting.ron` for `AssetServer` hot-reloading.
- **Direct Hierarchical Capture & 2-Pass Restore**: Replace the dummy-parameter `CaptureSnapshot`/`RestoreSnapshot` traits with direct typed methods (`WorldSave::capture` → `ShipSnapshot::capture` → `BlockSnapshot::capture` using $O(1)$ `&GridBlock` access) and a two-pass `WorldSave::restore` that maps saved ship `Uuid`s to spawned `Entity` IDs.
- **Unique Ship Instance Identity**: Separate `Ship.id` (unique per spawned ship instance, preserved across mutations and save/load) from `Ship.core_id` (the core block's `BlockPlacement.id` in `ShipGrid`) so multiple ships spawned from the same `ShipDesign` never collide on `Uuid`.
- **Session-Independent Weapon Cooldowns**: Replace absolute `time.elapsed_secs()` timestamps in `SingleShootPatternState`, `BurstShootPatternShootingState`, and `LaserBeamPatternShootingState` with relative remaining/elapsed seconds (`cooldown_remaining`, `barrel_delay_remaining`, `elapsed`) so weapon cooldowns work across sessions.
- **BREAKING: WorldSave v4 Full Snapshot Fidelity (`SAVE_SCHEMA_VERSION = 4`)**:
  - Persist ship cargo (`cargo_used`, `cargo_items`) on `ShipSnapshot`.
  - Persist battery `Charge`, weapon `TurretCharge`, and turret barrel rotation (`TurretSprite` angle) on `BlockSnapshot`.
  - Persist full `AiRuntimeSave` (`AiBehaviorSave` with target `Uuid`s, `spawn_position`, `rule_cooldowns`, and `timer_secs`).
  - Persist `ControlledShip`, `CameraTarget`, and `ControlMode` by ship `Uuid` on `WorldSave`.
  - Restore item drop `Sprite`s (rarity color and `config.0.drop_size`) and restore `BattlePhase::Ended` when `save.battle.outcome.is_some()`.

## Capabilities

### New Capabilities

### Modified Capabilities
- `world-save`: Upgrade `SAVE_SCHEMA_VERSION` to `4`, modularize `core/src/save/` into `battle`, `campaign`, `design`, and `document`, add `ShipCatalog` resource caching, separate `Ship.id` from `Ship.core_id`, use relative weapon pattern timers, and persist cargo, battery/turret charges, barrel angles, full AI runtime + target UUIDs, pilot/camera targets, and visible drop sprites.
- `campaign-progression`: Back `CampaignSave` and active battle save availability (`BattleSaveState`) with in-memory Bevy `Resource`s synchronized on campaign/battle save and reset events.
- `scene-lifecycle`: Restore `ControlledShip`, `CameraTarget`, `ControlMode`, and `BattlePhase::Ended` (when an outcome is present) during `SceneState::Loading` → `SceneState::Game` snapshot restoration.

## Impact

- `core/src/save/`: Replaced flat `catalog.rs`, `io.rs`, `schema.rs`, `snapshot.rs` with `battle/`, `campaign/`, `design/`, and `document.rs`.
- `core/src/content/ship/`: `Ship` gains `core_id: Uuid` alongside unique instance `id: Uuid`; `mutation.rs` preserves `ship.id`.
- `core/src/content/block/` & `core/src/content/bullet/`: Weapon pattern state components use relative remaining/elapsed seconds; turret barrel rotation and power/turret charges are restored.
- `core/src/mission/` & `core/src/scene/`: `home.rs`, `pause.rs`, `setting.rs`, `editor/`, and `mission/` read `ShipCatalog`, `CampaignSave`, and `BattleSaveState` resources and emit save/load/design messages.
- `docs/save.md`: Updated persistence specification to reflect `SAVE_SCHEMA_VERSION = 4` and the new module/resource architecture.
