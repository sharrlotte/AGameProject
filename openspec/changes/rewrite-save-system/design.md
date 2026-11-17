## Context

The persistence layer in `core/src/save/` currently mixes RON document I/O (`document.rs`), ship design file operations (`catalog.rs`), world snapshot schema (`schema.rs`), world capture/restore (`snapshot.rs`), and Bevy save/load systems (`io.rs`) in a flat directory. Several structural and gameplay defects exist:
- `BlockSnapshot::capture` scans every entity in the `World` (`world.iter_entities()`) for every block ($O(N \times M)$), and because `ShipDesign::spawn` reuses template `BlockPlacement.id`s across ships spawned from the same design, `placement_for` and `Ship.id` collide when multiple ships share a design.
- `CaptureSnapshot` and `RestoreSnapshot` force `WorldSave`, `ShipSnapshot`, and `BlockSnapshot` into one trait signature using dummy parameters (`Entity::PLACEHOLDER`, `_team`, `_parent`).
- `ShipCatalog::load_or_seed()`, `load_campaign()`, and `has_battle_save()` execute synchronous disk I/O directly from UI and mission setup systems instead of reading Bevy `Resource`s.
- Weapon pattern states (`SingleShootPatternState`, `BurstShootPatternShootingState`, `LaserBeamPatternShootingState`) store absolute `time.elapsed_secs()` timestamps, which freeze turrets when a save is loaded in a new session where `elapsed_secs` starts near `0.0`.
- `WorldSave` v3 omits ship cargo (`cargo_used`, `cargo_items`), battery `Charge`, weapon `TurretCharge`, turret barrel rotation, `ControlledShip` / `CameraTarget` / `ControlMode`, and full `AiRuntime` (`AiBehavior` parameters/targets, `spawn_position`, `rule_cooldowns`).
- `WorldSave::restore` spawns item drops without `Sprite` components and always sets `BattlePhase::Active` even when `save.battle.outcome` is `Some(...)`.

## Goals / Non-Goals

**Goals:**
- Restructure `core/src/save/` into feature-grouped submodules (`battle/`, `campaign/`, `design/`, and `document.rs`) adhering to the `mod.rs`, `component.rs`, `system.rs`, `lib.rs` pattern in `AGENTS.md`.
- Cache `ShipCatalog`, `CampaignSave`, and `BattleSaveState` as Bevy `Resource`s seeded/loaded once at startup and updated via messages/events.
- Replace `CaptureSnapshot` / `RestoreSnapshot` traits with direct hierarchical `capture` and two-pass `restore` methods with $O(1)$ `GridBlock` lookup.
- Separate `Ship.id` (unique ship instance `Uuid`) from `Ship.core_id` (core block `BlockPlacement.id`) so every spawned ship has a unique identity.
- Convert weapon pattern cooldowns/timers from absolute `elapsed_secs` timestamps to session-independent relative remaining/elapsed seconds.
- Upgrade `SAVE_SCHEMA_VERSION` to `4` with full snapshot fidelity for cargo, power/weapon charges, turret barrel rotation, AI runtime & target `Uuid`s, pilot/camera targets, drop sprites, and battle outcome phase restoration.

**Non-Goals:**
- Changing `GameSetting` (`assets/config/setting.ron`) persistence or removing its `AssetServer` hot-reload bridge.
- Changing `parse_document` or `remove_with_backup` in `document.rs` / `io.rs`.
- Supporting backward migration from `v3` `battle.ron` files (hard cutover to `SAVE_SCHEMA_VERSION = 4`).

## Decisions

### 1. Feature-Grouped `core/src/save/` Layout
- **Choice**: Split `core/src/save/` into:
  - `save/mod.rs`: `SavePlugin` registering `DesignSavePlugin`, `CampaignSavePlugin`, and `BattleSavePlugin`.
  - `save/document.rs`: `RonDocument`, `DocumentError`, `parse_document`, `read_document`, `write_document`, `atomic_write`, and `ron_document!`.
  - `save/design/`: `mod.rs`, `component.rs`, `system.rs`, `lib.rs` owning `ShipCatalog` (`Resource`), starter seeding, and design save/load messages.
  - `save/campaign/`: `mod.rs`, `component.rs`, `system.rs`, `lib.rs` owning `CampaignSave` (`Resource`), mission completion recording, and campaign reset.
  - `save/battle/`: `mod.rs`, `component.rs`, `system.rs`, `lib.rs` owning `WorldSave` (`SAVE_SCHEMA_VERSION = 4`), `BattleSaveState` (`Resource`), hierarchical capture, and two-pass restore.
- **Rationale**: Enforces single responsibility and `AGENTS.md` module rules while keeping each file focused and well under the 300-line threshold.

### 2. Direct Hierarchical Capture & Two-Pass Restore
- **Choice**:
  - `WorldSave::capture(ctx: &CaptureContext) -> Self` iterates ships and calls `ShipSnapshot::capture(world, entity_ref)`.
  - `ShipSnapshot::capture(world: &World, ship: EntityRef) -> Option<Self>` reads `ShipGrid` on the ship entity and calls `BlockSnapshot::capture(world, grid_block)` for each `&GridBlock` in $O(1)$.
  - `WorldSave::restore` runs in two passes:
    1. Spawn all ships with their saved `Ship.id`, apply `BlockSnapshot`s (health, ammo, turret state, barrel rotation, battery `Charge`, `TurretCharge`, weapon pattern states), restore ship cargo (`cargo_used`, `cargo_items`), and record `ship_map: HashMap<Uuid, Entity>`.
    2. Resolve `AiBehaviorSave` target `Option<Uuid>` into `Option<Entity>` via `ship_map` to insert full `AiRuntime` (`current`, `spawn_position`, `rule_cooldowns`, `think_timer`), and restore `ControlledShip`, `CameraTarget`, and `ControlMode`.
- **Rationale**: Eliminates `Entity::PLACEHOLDER` hacks, removes the $O(N \times M)$ entity scan, and cleanly resolves cross-entity references (`Entity` IDs cannot be serialized directly, whereas unique ship `Uuid`s map deterministically in pass 2).

### 3. Unique Ship Instance `Uuid` (`Ship.id` vs `Ship.core_id`)
- **Choice**: `Ship` stores `pub id: Uuid` (unique per ship instance, generated via `Uuid::now_v7()` on fresh `ShipDesign::spawn` or restored from `ShipSnapshot.id`) and `pub core_id: Uuid` (the `BlockPlacement.id` of the core block in `ShipGrid`). `has_core` checks `grid.block(ship.core_id).is_some()`, and `apply_ship_mutations` preserves the existing `ship.id` when rebuilding `Ship` after block mutations.
- **Rationale**: Guarantees every ship in a battle has a unique `Ship.id` even when multiple ships are spawned from the same `ShipDesign` template.

### 4. Relative Weapon Pattern Timers
- **Choice**:
  - `SingleShootPatternState { pub cooldown_remaining: f32 }`
  - `BurstShootPatternShootingState::Shooting { current_barrel: usize, barrel_delay_remaining: f32, current_amount: i8 }` and `Cooldown { cooldown_remaining: f32 }`
  - `LaserBeamPatternShootingState::Shooting { elapsed: f32 }` and `Cooldown { cooldown_remaining: f32 }`
- **Rationale**: Ticking relative timers by `time.delta_secs()` in `FixedUpdate` makes weapon state completely independent of session uptime (`time.elapsed_secs()`), so serialized weapon states resume immediately after loading in a new session.

### 5. In-Memory Resource Caching
- **Choice**: Initialize `ShipCatalog`, `CampaignSave`, and `BattleSaveState` (`pub has_save: bool`) as Bevy `Resource`s at startup. Disk writes update the corresponding resource in-place. `GameSetting` remains in `assets/config/setting.ron`.
- **Rationale**: UI (`Home`, `Settings`, `ShipEditor`) and `MissionPlugin` read ECS resources in $O(1)$ memory without touching the filesystem on frame updates or scene transitions.

## Risks / Trade-offs

- **[Breaking Save Schema (`v3` → `v4`)]** → Existing `v3` `battle.ron` files will be rejected with `DocumentError::Version { expected: 4, found: 3 }`, which surfaces cleanly via `AppNotice` and returns the player to `SceneState::Home`.
- **[Deferred Entity Commands During Restore]** → In `WorldSave::restore`, `ShipDesign::spawn_with_id` returns the spawned `ship_entity` `Entity` ID immediately from `commands.spawn(...).id()`, so `ship_map: HashMap<Uuid, Entity>` can be populated and consumed within the same system before command flush.
