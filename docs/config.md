# Config System Spec

## Goals

- Hardcoded game balance data lives in `.ron` files under `core/assets/config/` with hot
  reload (bevy AssetServer + `file_watcher`, dev-only).
- Game settings (`GameSetting`: volume, bloom, `ai_level`, `keys`) are file-backed `.ron` with hot reload
  and save-on-change. `ai_level` clamps `1..=10` (default `5`); `keys` holds `rotate_ccw`/`rotate_cw` (defaults `KeyQ`/`KeyE`) and is
  merged field-by-field with defaults when absent.
- Single source of truth: the `.ron` files are the only definition of balance data; this
  doc describes the design, not the current values.

## Non-goals

- Save format stays RON (`save/document.rs`).
- Structural/layout constants stay in code: `GRID_SIZE`, `SMALL_GRID_SIZE`, `LAYER_*`,
  `AUDIO_SCALE`, `FLASH_OVERLAY_COLOR`, `VIEW_HEIGHT`.
- Chunk streaming tuning stays in code as compile-time constants in `core/src/constant.rs`:
  `CHUNK_SIZE`, `LOAD_RADIUS`, `DESPAWN_RADIUS`, `STREAM_INTERVAL_SECS`. There is no
  `chunk.ron` config, `ChunkConfig` asset/resource, hot-reload bridge, or runtime validation;
  invariants are guarded by unit tests only.
- Camera feel (zoom speed, smooth speed, boost, pan speed) and beam/laser feel (laser
  intensity `*10.0`, bulge `0.5` ease floor, clamp `0.4..1.0`) stay in code.
- Items (`ItemDef`) out of scope; can follow the same pattern later.
- Hot reload applies to **new spawns only**; live entities keep the values they spawned with.
- No embedded `include_str!` fallback for game balance data.

## Decisions (approved)

- Async bevy `AssetServer` load + custom RON `AssetLoader`; `file_watcher` enabled only via
  the `dev` cargo feature (production: no watcher; settings apply live via UI, external
  edits need restart).
- Boot gate: `ConfigState::Loading -> Ready`; app boots with `SceneState::Loading` (plain
  black screen, nothing spawned); on Ready, transitions to `SceneState::Home`.
- Boot failure policy: missing/unparseable `game.ron`/`block.ron` moves `ConfigState` to
  `Error(ConfigError)` and renders a full-screen error — the game does not panic.
  Hot-reload failure = `error!` log + keep last good values (no crash mid-session).
  `setting.ron` missing/corrupt at boot = fall back to defaults + `warn!`.
- `setting.ron` lives in `core/assets/config/setting.ron` (inside the asset root so it is
  watched by the same pipeline). Its `keys` section is optional and merged with defaults.
- `game.ron` is a single flat file; `deny_unknown_fields`.

## Files

| Path (relative to asset root) | Type | Boot failure | Reload failure |
|---|---|---|---|
| `config/base/game.ron` | `GameConfig` asset | error screen | keep last good + error log |
| `config/base/block.ron` | `BlocksConfig` asset | error screen | keep last good + error log |
| `config/base/ai.ron` | `AiConfig` asset | error screen | keep last good + error log |
| `config/base/minimap.ron` | `MinimapConfig` asset | error screen | keep last good + error log |
| `config/setting.ron` | `GameSetting` asset | defaults + warn | keep last good + error log |
| `config/quest/quests.ron` | `QuestCatalogDef` asset | defaults to empty catalog + error log | keep last good + error log |
| `config/quest/generation.ron` | `QuestGenConfig` asset | defaults + warn | keep last good + error log |
| `config/base/skirmish.ron` | `SkirmishDef` asset | warning + return home | keep last good + error log |

Quest assets are loaded by `QuestPlugin` with `RonAssetLoader` and applied once config is ready,
then hot-reloaded through an `AssetEvent` bridge (see quests.md). Hot reload of `quests.ron`
replaces authored entries only; generated entries are preserved.

No RON config exists for chunk streaming: its tuning is compile-time constants in `core/src/constant.rs`, not a hot-reloadable asset.

Asset root resolution for settings write-back: `BEVY_ASSET_ROOT` env var, else `assets`
relative to CWD (same default bevy uses).

## Module Layout

`core/src/config/` owns all configuration definitions, schemas, and conversions:

```
core/src/config/
├── mod.rs               // ConfigPlugin, ConfigState, ConfigError
├── loader.rs            // RonAssetLoader<A: Asset + DeserializeOwned + Validate<Context = ()>>
├── setting.rs           // GameSetting, KeySettings
├── game/
│   ├── mod.rs
│   ├── game.rs          // GameConfig, GameConfigValue, bridge
│   └── fleet.rs         // DefaultFleetEntry, fleet spacing validation
├── block/
│   ├── mod.rs           // BlocksConfig, blocks_bridge
│   ├── def.rs           // BlockDefConfig, into_def
│   └── feature/         // FeatureConfig enum and feature modules
│       ├── mod.rs
│       ├── turret.rs    // TurretComponentConfig, ShootPatternConfig
│       ├── shield.rs    // ShieldConfig, ShieldShapeConfig
│       ├── thruster.rs  // ThrusterConfig
│       ├── power.rs     // Generator, Battery, Consumer, Booster
│       ├── ammo.rs      // AmmoConfig
│       ├── texture.rs   // TextureConfig
│       └── storage.rs   // StorageConfig
├── ai/
│   ├── mod.rs           // AiConfig, AiConfigValue, ai_config_bridge
│   └── types.rs         // SkillCurves, SkillUnlocks, IntelBounds, tuning structs
└── minimap/
    └── mod.rs           // MinimapConfig, MinimapConfigValue, minimap_config_bridge
```

## Loader design

### Custom RON asset loader (`config/loader.rs`)

- `RonAssetLoader` implements `AssetLoader` for any `A: Asset + DeserializeOwned + Validate<Context = ()>`.
- Executes `asset.validate(&())` immediately after byte deserialization. Any validation error returns
  `io::ErrorKind::InvalidData`, causing the asset load to fail with exact file and reason.
- Parse or validation failure moves `ConfigState` to `Error` on boot and logs an error on hot reload.
- `ron` crate added as a direct dependency (already in the lockfile via bevy).
- `.ron` extension registered.

### Assets and resources

Each `.ron` asset has a matching runtime resource; a bridge system copies asset → resource
on `AssetEvent::LoadedWithDependencies` / `Modified`:

| Asset (`Asset, TypePath, Deserialize, Clone, PartialEq`) | Resource | Consumers |
|---|---|---|
| `GameConfig` | `GameConfigValue(pub GameConfig)` | systems replacing `constant::X` |
| `BlocksConfig` | rebuilds `BlockRegistry` | existing registry consumers |
| `AiConfig` | `AiConfigValue(pub AiConfig)` | `content/ship/ai/*`, `mission/state.rs`, `ship/system.rs` |
| `MinimapConfig` | `MinimapConfigValue(pub MinimapConfig)` | `scene/game/minimap/*` |
| `GameSetting` | `GameSetting` (existing resource + `ai_level` + `Serialize`/`PartialEq` + `Asset`/`TypePath`) | `scene/setting.rs`, `mission/state.rs` |

- Bridge on `Modified`: `GameConfig` → overwrite `GameConfigValue`; `BlocksConfig` →
  rebuild `BlockRegistry` map wholesale; `GameSetting` → `set_if_neq` clone into resource
  (no `is_changed` churn).
- Reload failure: bevy logs the loader error; bridge does nothing; last good values stay.
  Matches approved policy.
- Shared unchanged helper `reload_if_present` / per-asset systems, all in `config/`.

### Boot sequence (`ConfigPlugin`)

1. `init_state::<ConfigState>()`, `ConfigState::Loading` default.
2. `SceneState::Loading` variant, `#[default]`; black screen, zero spawns; `ConfigPlugin`
   sets `NextState<SceneState> = Home` in the same system that sets `ConfigState::Ready`.
3. `Startup`: `asset_server.load("config/base/game.ron")`, `"config/base/block.ron"`,
   `"config/base/ai.ron"`, `"config/base/minimap.ron"`, `"config/setting.ron"`; handles stored
   in `ConfigHandles` resource.
4. `Update` while `Loading`: poll `LoadState` of game + blocks handles:
   - `Failed` / `Error` → move `ConfigState` to `Error(ConfigError)`; the render path
     shows the full-screen error. No panic.
   - both `Loaded` → build `BlockRegistry` from `BlocksConfig`, insert `GameConfigValue`,
     insert `GameSetting` resource (defaults if settings asset failed/missing + warn),
     `ConfigState → Ready`, `SceneState → Home`.
5. `Update` while `Ready`: bridge systems react to `AssetEvent::{Modified,
   LoadedWithDependencies}`; dev-only `file_watcher` drives the events.

### Settings (`GameSetting`)

- `GameSetting` (existing struct) gains `Serialize`, `Deserialize`, `PartialEq`, `Asset`,
  `TypePath` and keeps `Resource`. Single schema = single source of truth.
- `Update`: if `Res<GameSetting>.is_changed() && !from_reload` (flag set/cleared by the
  bridge) → serialize to `<asset_root>/config/setting.ron` (create parent dir), pretty
  RON.
- Bridge on settings reload assigns with `set_if_neq` → identical content marks nothing
  changed → no save echo loop (write → reload → equal → no write).
- `apply_settings`, `sync_settings_to_ui`, UI handlers unchanged.

## `game.ron` and `block.ron`

The schema is owned by the files themselves (`deny_unknown_fields`); the values in this
document are illustrative, not normative:

- `config/base/game.ron` — flat map of scalars: damping, thrust/turn factors, drop tuning,
  `max_ships`, zoom bounds, missile/bullet/laser feel, AI tuning,
  `debris_lifetime_min_seconds` / `debris_lifetime_max_seconds`, and
  `auto_save_interval_secs`. The debris range is validated at load: both bounds must be
  positive and the maximum must be at least the minimum.
- `config/base/block.ron` — `blocks:` map keyed by plain `String` id; each entry: `name`,
  `block_group`, `mass`, `health`, `cost`, `shape` (`ShapeSpec`), `features` (`Texture`,
  `Turret`, `Generator`, `Battery`, `Consumer`, `Booster`, `Thruster`, `Storage`), `drops`.
- `config/base/minimap.ron` — `MinimapConfig`: `size_px`, `margin_px`,
  `fallback_world_radius`, `radar_scale_factor`, `refresh_secs`, `dot_size_px`,
  `lost_flash_count`, `lost_flash_secs`, `glow_blur_px`, `max_markers`. Validation requires
  positive `size_px`/`fallback_world_radius`/`radar_scale_factor`/`refresh_secs`/`dot_size_px`/
  `lost_flash_secs`/`glow_blur_px`, non-negative `margin_px`, `lost_flash_count >= 1`, and
  `max_markers >= 1`.

### Shape spec (`ShapeSpec`)

`shape` is a single value, with no separate `size` or `voxels` field:

```ron
shape: Square(3),
shape: Rect((3, 5)),
shape: Triangle(base: Right, size: (3, 3)),
shape: Triangle(base: Isosceles, size: (5, 3), xform: (rotate: R90, flip_x: true)),
shape: Rows(art: ["###", "#.#", "###"]),
```

- `Square(n)` is an `n×n` grid; `Rect((w, h))` is a filled `w×h` grid.
- `Triangle` bases are `Right` and `Isosceles`, rasterized by an integer center-in rule;
  its rasterized bounding box must equal the declared `size`.
- `Rows` uses `#` for filled and `.` for empty. Rows must be rectangular, contain at least
  one `#`, use only those two characters, and have no empty border row or column.
- `Triangle` and `Rows` carry `xform: (rotate, flip_x, flip_y)`, defaulting to identity.

**Transform order:** base cells → definition `Xform` (rotate CCW, then `flip_x`, then
`flip_y`) → normalize to min `(0,0)` → placement `Rot90` (CCW) → normalize again. `size`
is recomputed after each normalization, so `Rect((3,5))` at `R90` has size `(5,3)`.

**Baking and validation:** each spec is validated and baked once at config load (and on
hot reload) by `TryFrom<&ShapeSpec> for BakedShape`. Baking rejects non-positive
dimensions, malformed `Rows`, a triangle whose raster does not span its declared size,
zero cells, disconnected cells, and a triangle with no buildable convex hull. Failures
name the block key and move `ConfigState` to `Error` on boot; on hot reload the last good
registry is kept and the error is logged. `BlockDef` holds only the read-only baked form:
four `(Footprint, Hitbox)` pairs keyed by placement `Rot90`, exposed as slices/values.

**Art convention:** art is authored once for the base, untransformed shape; its alpha
matches the true shape (e.g. smooth triangle edges). Cells govern grid occupancy and
connection only. The `Visual` child applies definition rotation and flips as signed scale;
the block entity carries only placement rotation so colliders never see negative scale.

Usage sites read `Res<GameConfigValue>` / `BlockRegistry` instead of `constant::X`.
`constant.rs` keeps grid/layer/color/audio/camera-design constants and the chunk
streaming tuning constants (`CHUNK_SIZE`, `LOAD_RADIUS`, `DESPAWN_RADIUS`,
`STREAM_INTERVAL_SECS`); other gameplay constants moved into `game.ron`.

- Serde mirror types in `config/block.rs` for `Feature` (`FeatureConfig`) and any struct
  holding `Handle`s (`Texture`, `TurretComponent` image, `Ammo` icon) with
  `image`/`sound` as asset-relative path strings (same `format!("texture/{image}")` /
  `format!("audio/{sound}")` convention as today).
- Registry keys are plain `String` ids: `block.ron` is keyed by id and `BlockRegistry`
  exposes `get_symbol(key)`. `ItemDrop.item_type` is a `String`.
- `Vec2`/`IVec2`: bevy `serialize` feature provides serde support.
- `BlockDefConfig::into_def(assets: &AssetServer, key) -> Result<BlockDef, ShapeSpecError>`
  bakes the shape spec (validating it) and loads handles; conversion failures include the
  block key and are routed to `ConfigState::Error`.
- Turret shoot sound (`BurstShootPattern.shoot_sound`) stays a path string in config →
  resolved at conversion.
- `PreStartup` block registration is gone; the registry is built in the boot sequence.

## Module layout

```
core/src/config/
  mod.rs       ConfigPlugin, ConfigState, ConfigHandles, re-exports
  loader.rs    RonAssetLoader<A: Asset + DeserializeOwned>
  game.rs      GameConfig asset + GameConfigValue resource + bridge system
  block.rs     BlocksConfig asset + BlockDefConfig/FeatureConfig mirrors + into_def + bridge/rebuild
  setting.rs   GameSetting, bridge + save-on-change systems
```

The minimap asset lives with its owning feature: `scene/game/minimap/config.rs`
(`MinimapConfig` asset, `MinimapConfigValue` resource, validation, hot-reload bridge).

## Touch points

- `core/Cargo.toml`: `ron = "0.12"`; `dev` feature adds `"bevy/file_watcher"`.
- `main.rs`: `ConfigPlugin`.
- `resource.rs`: `SceneState::Loading` variant, `#[default]`.
- `constant.rs`: other gameplay constants removed; grid/layer/color/audio and chunk streaming tuning constants kept.
- `scene/setting.rs`: `GameSetting` derives extended; `sync/apply` read
  `Res<GameSetting>` as today.
- `content/block/system.rs`: `register_default_blocks` deleted; `handle_drop` reads
  `Res<GameConfigValue>`.
- `content/block/feature/thruster/`, `content/ship/`, `scene/game.rs`,
  `content/bullet/system.rs`, `content/damage/system.rs`: constants → `Res<GameConfigValue>`.

## Verification

- Single `cargo check` (compile is slow).
- Boot: menu appears only after config Ready (black screen otherwise); delete/corrupt
  `game.ron` → error screen with a clear message (no panic, no crash).
- Edit `block.ron` health while running (dev build) → newly spawned block has new health,
  existing unaffected.
- Change volume in UI → `assets/config/setting.ron` updated; edit it externally → volume
  applies within a frame (watcher).
- Save-on-change loop check: after UI change, no repeated writes/reload churn.

## Risks

- `Feature::spawn` threading of `GameConfigValue` touches several call sites; mechanical.
- Reload swap of `BlockRegistry` while spawning: acceptable (atomic map replacement; new
  spawns only).
- `SceneState::Loading` spawns nothing; exiting it despawns nothing (no entities to
  despawn). Cleanup of Game entities is owned by `DespawnOnExit(SceneState::Game)`.
- Settings write-back depends on CWD/`BEVY_ASSET_ROOT`; documented above.

## Out of scope (future)

- `items.ron`, `ships.ron`
- Live re-application to existing entities
- Non-dev hot reload

## Numeric Validation

Block config validation SHALL cover projectile, laser, and thruster numeric fields in addition to existing rules: projectile `speed >= 0`, `damage >= 0`, `lifetime >= 0`, `width > 0`, `length > 0`, `piercing >= 0` (bullet); laser `damage >= 0`, `width > 0`, `cooldown >= 0`, `duration > 0`; burst `amount >= 1`, `fire_rate > 0`; thruster `force >= 0`, `torque >= 0`. Invalid values SHALL be clamped with a warning naming the block, and the config error screen SHALL report structural validation failures.
