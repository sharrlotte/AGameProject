# Config System Spec

## Goals

- Move hardcoded game balance data into `.ron` files under `game/core/assets/config/` with hot reload (bevy AssetServer + `file_watcher`, dev-only).
- Game settings (`GameSetting`: volume, bloom) become file-backed `.ron` with hot reload and save-on-change.
- Single source of truth: `.ron` files are the only definition of balance data.

## Non-goals

- Save format stays JSON (`save/io.rs`), schematics unchanged.
- Structural/layout constants stay in code: `GRID_SIZE`, `SMALL_GRID_SIZE`, `LAYER_*`, `AUDIO_SCALE`, `FLASH_OVERLAY_COLOR`.
- Camera feel (zoom speed, smooth speed, boost, pan speed) and beam/laser feel (laser intensity `*10.0`, bulge `0.5` ease floor, clamp `0.4..1.0`) stay in code.
- Items (`ItemDef`) out of scope; can follow the same pattern later.
- Hot reload applies to **new spawns only**; live entities keep the values they spawned with.
- No embedded `include_str!` fallback (startup failure = panic, see below).

## Decisions (approved)

- Async bevy `AssetServer` load + custom RON `AssetLoader`; `file_watcher` enabled only via the `dev` cargo feature (production: no watcher; settings apply live via UI, external edits need restart).
- Boot gate: new `ConfigState::Loading -> Ready`; app boots with `SceneState::Loading` (plain black screen, nothing spawned); on Ready transitions to `SceneState::Home`.
- Startup failure policy: missing/unparseable `game.ron`/`block.ron` at boot = panic with a clear message. Hot-reload failure = `error!` log + keep last good values (no crash mid-session). `setting.ron` missing/corrupt at boot = fall back to defaults + `warn!`.
- `setting.ron` lives in `game/core/assets/config/setting.ron` (inside the asset root so it is watched by the same pipeline).
- `game/assets/config/{base,combat}` (empty, wrong root) are deleted.
- `game.ron` is a single flat file; `deny_unknown_fields`.

## Files

| Path (relative to asset root) | Type | Boot failure | Reload failure |
|---|---|---|---|
| `config/base/game.ron` | `GameConfig` asset | panic | keep last good + error log |
| `config/base/block.ron` | `BlocksConfig` asset | panic | keep last good + error log |
| `config/setting.ron` | `GameSetting` asset | defaults + warn | keep last good + error log |

Asset root resolution for settings write-back: `BEVY_ASSET_ROOT` env var, else `assets` relative to CWD (same default bevy uses).

## Loader design

### Custom RON asset loader (`config/ron.rs`)

- `RonAssetLoader` implementing `AssetLoader` for any `A: Asset + DeserializeOwned`; returns `Err` on parse failure (bevy logs it; old asset kept on reload; load marked Failed at boot).
- `ron` crate added as a direct dependency (already in the lockfile via bevy).
- `.ron` extension registered.

### Assets and resources

Each `.ron` asset has a matching runtime resource; a bridge system copies asset → resource on `AssetEvent::LoadedWithDependencies` / `Modified`:

| Asset (`Asset, TypePath, Deserialize, Clone, PartialEq`) | Resource | Consumers |
|---|---|---|
| `GameConfig` | `GameConfigValue(pub GameConfig)` | systems replacing `constant::X` |
| `BlocksConfig` | rebuilds `BlockRegistry` | existing registry consumers |
| `GameSetting` | `GameSetting` (existing resource, unchanged shape + `Serialize`/`PartialEq` + `Asset`/`TypePath` derives for the asset side) | `scene/setting.rs`, unchanged |

- Bridge on `Modified`: `GameConfig` → overwrite `GameConfigValue`; `BlocksConfig` → rebuild `BlockRegistry` map wholesale; `GameSetting` → `set_if_neq` clone into resource (no `is_changed` churn).
- Reload failure: bevy logs the loader error; bridge does nothing; last good values stay. Matches approved policy.
- Shared unchanged helper `reload_if_present` / per-asset systems, all in `config/`.

### Boot sequence (`ConfigPlugin`)

1. `init_state::<ConfigState>()`, `ConfigState::Loading` default.
2. `SceneState::Loading` added as variant, `#[default]`; black screen, zero spawns; `ConfigPlugin` sets `NextState<SceneState> = Home` in the same system that sets `ConfigState::Ready`.
3. `Startup`: `asset_server.load("config/base/game.ron")`, `"config/base/block.ron"`, `"config/setting.ron"`; handles stored in `ConfigHandles` resource.
4. `Update` while `Loading`: poll `LoadState` of game + blocks handles:
   - `Failed` / `Error` → `panic!("config/base/game.ron failed to load: ...")` (startup hard fail).
   - both `Loaded` → build `BlockRegistry` from `BlocksConfig`, insert `GameConfigValue`, insert `GameSetting` resource (defaults if settings asset failed/missing + warn), `ConfigState → Ready`, `SceneState → Home`.
5. `Update` while `Ready`: bridge systems react to `AssetEvent::{Modified, LoadedWithDependencies}`; dev-only `file_watcher` drives the events.

### Settings (`GameSetting`)

- `GameSetting` (existing struct) gains `Serialize`, `Deserialize`, `PartialEq`, `Asset`, `TypePath` and keeps `Resource`. Single schema = single source of truth.
- `Update`: if `Res<GameSetting>.is_changed() && !from_reload` (flag set/cleared by the bridge) → serialize to `<asset_root>/config/setting.ron` (create parent dir), pretty RON.
- Bridge on settings reload assigns with `set_if_neq` → identical content marks nothing changed → no save echo loop (write → reload → equal → no write).
- `apply_settings`, `sync_settings_to_ui`, UI handlers unchanged.

## `game.ron` schema

Flat, all fields required, `deny_unknown_fields`:

```ron
(
    thrust_power_factor: 50.0,
    turn_speed_factor: 9000.0,
    linear_damping: 0.9,
    angular_damping: 0.999,
    drop_size: 10.0,
    drop_despawn_in_seconds: 300.0,
    drop_collect_radius: 1000.0,
    max_ships: 16,
    min_zoom: 0.2,
    max_zoom: 30.0,
    missile_accel_time: 0.35,
    missile_max_turn: 2.0,
    bullet_fade_time: 0.25,
    damage_number_lifetime: 1.5,
    ai_flee_distance: 1000.0,
    ai_wander_patrol_radius_multiplier: 10.0,
    ai_group_range_multiplier: 2.0,
    ai_min_range_factor: 0.9,
    ai_wander_attack_chance: 0.5,
    ai_wander_guard_chance: 0.5,
)
```

Usage sites switch from `constant::X` to `Res<GameConfigValue>`:

- thruster insert (`THRUST_POWER_FACTOR`, `TURN_SPEED_FACTOR`) — threaded via `Feature::spawn` → `Thruster::insert`
- damping setup (`LINEAR_DAMPING`, `ANGULAR_DAMPING`) + `stop_distance`
- drop spawn size/despawn (`DROP_SIZE`, `DROP_DESPAWN_IN_SECONDS`), collect radius (`DROP_COLLECT_RADIUS`)
- zoom clamp (`MIN_ZOOM`, `MAX_ZOOM`), `MAX_SHIPS`, missile (`missile_accel_time`, `missile_max_turn`), laser fade (`FADE_TIME`), damage number float (`damage_number_lifetime`)
- AI (`AI_THINK_RATE_HZ` is a shared constant in `constant.rs`; `Flee` 1000.0 offset, Wander 0.5/0.5 chances, Regroup/Guard `radius*2.0`, `min_range*0.9`)

`constant.rs` keeps only grid/layer/color/audio constants; `ANGULAR_DAMPING`/`LINEAR_DAMPING` and the other moved constants are removed.

## `block.ron` schema

```ron
(
    blocks: {
        hull: (
            name: "Standard Hull",
            block_group: Base,
            mass: 10.0,
            health: 1000.0,
            cost: 5,
            shape: (size: (1, 1), voxels: [(position: (0, 0), connectable_faces: 255)]),
            features: [
                Texture(image: "block/surge-wall-large.png", offset: (0.0, 0.0), z: 10.0, animated: false),
            ],
            drops: [Item(item_type: "steel", amount: 10)],
        ),
        // ... one entry per block; the map key is a plain String block id
    },
)
```

- Serde mirror types in `config/block.rs` for `Feature` (`FeatureConfig`) and any struct holding `Handle`s (`Texture`, `TurretComponent` image, `Ammo` icon) with `image`/`sound` as asset-relative path strings (same `format!("texture/{image}")` / `format!("audio/{sound}")` convention as today).
- Registry keys are plain `String` ids: `block.ron` is keyed by id and `BlockRegistry` exposes `get(key: &str)`. `ItemDrop.item_type` is a `String`.
- `Vec2`/`IVec2`: bevy `serialize` feature provides serde support — verify during impl; else tuple mirrors.
- `BlockDefConfig::into_def(assets: &AssetServer) -> BlockDef` converts and loads handles.
- Turret shoot sound (`BurstShootPattern.shoot_sound`) stays a path string in config → resolved at conversion.
- `register_default_blocks` deleted; block values exist only in `block.ron`.
- `PreStartup` registration removed; registry built in the boot sequence instead.

## Module layout

```
game/core/src/config/
  mod.rs       ConfigPlugin, ConfigState, ConfigHandles, re-exports
  loader.rs    RonAssetLoader<A: Asset + DeserializeOwned> (renamed from ron.rs
               to avoid a name clash with the `ron` crate)
  game.rs      GameConfig asset + GameConfigValue resource + bridge system
  block.rs    BlocksConfig asset + BlockDefConfig/FeatureConfig mirrors + into_def + bridge/rebuild
  setting.rs  GameSetting (moved here), bridge + save-on-change systems
```

## Touch points

- `core/Cargo.toml`: add `ron = "0.12"`; `dev` feature adds `"bevy/file_watcher"`.
- `main.rs`: add `ConfigPlugin`; no other changes.
- `resource.rs`: `SceneState::Loading` variant, `#[default]`.
- `constant.rs`: gameplay constants removed (grid/layer/color/audio kept).
- `scene/setting.rs`: `GameSetting` derives extended; `sync/apply` read `Res<GameSetting>` as today.
- `content/block/system.rs`: `register_default_blocks` deleted; `handle_drop` reads `Res<GameConfigValue>`.
- `content/block/feature/thruster/component.rs`, `content/ship/{components,systems}.rs`, `scene/game.rs`, `content/bullet/system.rs`, `content/damage/system.rs`: constants → `Res<GameConfigValue>`.
- Empty `game/assets/config/` deleted.

## Verification

- Single `cargo check` (compile is slow).
- Boot: menu appears only after config Ready (black screen otherwise); delete/corrupt `game.ron` → panic with clear message; corrupt `setting.ron` → defaults + warn, game runs.
- Edit `block.ron` health while running (dev build) → newly spawned block has new health, existing unaffected.
- Change volume in UI → `assets/config/setting.ron` updated; edit it externally → volume applies within a frame (watcher).
- Save-on-change loop check: after UI change, no repeated writes/reload churn.

## Status (rewrite branch, Slice 3)

Implemented as specified, with these deltas (all spec, no notes):

- `RonAssetLoader<A>` lives in `config/loader.rs` (renamed from `ron.rs` to avoid a clash
  with the `ron` crate); `ron = "0.12.3"` added as a direct dependency.
- `GameSetting` moved fully into `config/setting.rs` with `Asset`/`TypePath` added;
  `scene/setting.rs` keeps the UI only.
- Boot gate implemented: `SceneState::Loading` is the new `#[default]`; `ConfigPlugin`
  transitions `Loading → Home` only after `game.ron` and `block.ron` are read.
- `block.ron` checked in under `assets/config/base/`; `game.ron` and
  `config/setting.ron` also checked in. Empty `game/assets/` deleted.
- `register_default_blocks` deleted; `register_default_items` stays (items deferred).
- Shared derives: `DropComponent`, `ShootingProjectile`, `SingleShootPattern`,
  `LaserBeamPattern`, `PullingBeamPattern`, `BlockShape`, `BlockVoxel`, `BlockGroup`,
  `ItemDrop`, `ItemType`, `Thruster`, `EnergyConsumer`, `EfficiencyProvider` and
  `TargetingStrategy` gained `Serialize`/`Deserialize`. `BurstShootPattern` stays
  runtime-only (`Handle<AudioSource>`); its `.ron` side is `BurstShootPatternConfig`
  with a path string.
- A subtlety: `setting.ron` lives in the watched folder, and the bridge assigns with
  `set_if_neq` — identical content marks nothing changed, so the save-on-change loop
  does not echo.
- Empty-name rule: no unit `.ron` fallback exists; boot panics with a clear message
  on missing/unparseable `game.ron`/`block.ron`.

## Risks

- `Feature::spawn` threading of `GameConfigValue` touches several call sites; mechanical.
- Reload swap of `BlockRegistry` while spawning: acceptable (atomic map replacement; new spawns only).
- `SceneState::Loading` must not confuse `clean_state_item` (no `SceneEntity` spawned for it) — verified by design, it only reacts to transitions away from states that had entities.
- Settings write-back depends on CWD/`BEVY_ASSET_ROOT`; documented above.

## Out of scope (future)

- `items.ron`, `ships.ron`
- Live re-application to existing entities
- Non-dev hot reload
