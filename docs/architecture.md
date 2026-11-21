# Architecture

## Workspace layout

```
AGameProject/                Cargo workspace (resolver = "3")
  core/                      single crate, bin target (host application)
    assets/                  assets (root of asset lookup; see assets.md)
    src/
      main.rs                plugin wiring, window, audio, fixed-timestep setup
      resource/              states + typed shared resources (`mod.rs`, `session.rs`)
      simulation.rs          named simulation-set ordering contract
      system/                camera, startup fleet load, debug scene audit
        mod.rs               module root, plugin declaration, debug gate
      constant.rs            global constants (grid, layers, chunk tuning)
      asset/                 asset check plugin (`dev`)
      content/               gameplay content (one module per domain)
      control/               player ship control & camera follow
      quest/                 authored & generated quests, HUD, progression
      scene/                 UI/state scenes
        editor/              editor module root (state, plugin wiring) + feature modules
          mod.rs             module root: declarations, re-exports, plugin
          {ui,input,grid,save,ship_list,ai_panel,fleet}.rs  editor feature modules
        {background,game,home,notice,pause,setting,widget}/  scene modules
          mod.rs             module root: declarations, re-exports, plugin
        layout.rs            shared UI layout constants
      ui/                    modal stack & shared UI state
      validate/              shared validation trait and assertion helpers
      config/                modular RON config definitions (block, game, ai, minimap, setting)
      save/                  persistence (`design/`, `world/`, `document.rs`)
        design/              ShipDesign definition, ShipCatalog, sandbox designs
        world/               SQLite save slots: component/store/sqlite/system, snapshot/ (block, ship, debris, world, sanitize, capture, apply)
      world/                 world spatial simulation (chunk partitioning, distance streaming)
        chunk/               spatial chunk persistence & streaming (mod.rs, component.rs, system.rs)
                             tuning consts live in constant.rs (CHUNK_SIZE, LOAD_RADIUS,
                             DESPAWN_RADIUS, STREAM_INTERVAL_SECS); no config.rs
```

`release` profile: `lto = "thin"`, `codegen-units = 1`, `opt-level = "s"`, stripped.
Dev profile optimizes dependencies, not the workspace crate.

## States

Two Bevy 0.20 state machines, plus `ModalStack` for overlays:

| State | Source | Variants | Default |
|---|---|---|---|
| `SceneState` | `ui/state.rs` | `Loading`, `Home`, `Game`, `ShipEditor` | `Loading` |
| `ConfigState` | `config/mod.rs` | `Loading`, `Ready`, `Error(ConfigError)` | `Loading` |

`ConfigState` is registered inside `ConfigPlugin`; `SceneState` is registered in `main.rs`.
Scene-scoped entities are tagged with Bevy's `DespawnOnExit(SceneState::...)` (`Game`, `Home`, `ShipEditor`). Modal overlays (`Modal::Pause`, `Modal::Settings`, `Modal::SlotPicker`) are managed by the `ModalStack` resource with `ModalRoot(Modal)` and despawn when closed. A debug-only audit in `system/mod.rs` samples root entities every five seconds and warns about any that lack a `DespawnOnExit` tag.

Config load failures move `ConfigState` to `Error(ConfigError)` and show a
full-screen error instead of panicking. Skirmish/load failures post an
`AppNotice`. Save slots are single SQLite files
validated by `PRAGMA user_version`; the Home sandbox design
catalog remains versioned RON owned by `save/document.rs`, and `GameSetting`
persists to `assets/config/setting.ron`.

## Typed shared resources

| Resource | Contents |
|---|---|
| `DeployPayload` | `fleet: Vec<ShipDesign>` — starter/deployed fleet for skirmish & respawn |
| `CameraTarget` | `following: Option<Entity>` — camera-follow ship |
| `ControlledShip` | `controlling: Option<Entity>` — ship receiving player input |
| `ControlMode` | `Pilot` (WASD flies the controlled ship, camera follows) or `FreeLook` (WASD pans, thrust paused); `F` toggles, middle-mouse drags the camera in either mode |
| `WorldContext` | `QuickBattle { store }`, `SaveSlot { slot_id, path, store }`, `Restoring { slot_id, path, store }` — owns active session identity and `Box<dyn WorldStore>` |
| `GameConfigValue` | `GameConfig` from `game.ron` (damping, thrust, AI tuning, drops, `auto_save_interval_secs`) |
| `StreamingTimer` | `Timer` controlling periodic chunk streaming intervals |
| `SessionClock` | accumulated play time across save/load |
| `GameRng` | `seed: u64` — xorshift64* PRNG seeded from `world_meta.seed` |
| `QuestTracker` | active/completed `QuestRecord`s |
| `ShipCatalog` | cached sorted `Vec<ShipDesign>` |
| `InspectedShip` | `Option<Entity>` — ship shown in `ShipInspectorRoot` |
| `DesignCatalogSource` | `ActiveWorldDb` (slot) or `HomeSandbox` (Home/QuickBattle) |
| `SlotPickerUi` | slot list, selection, delete confirmation, new-world name buffer |
| `EditorReturnTarget` | `Home` or `Game` for the Ship Editor's back button |
| `ModalStack` | active modal overlay stack (`Pause`, `Settings`, `SlotPicker`) |

Chunk tuning (`CHUNK_SIZE`, `LOAD_RADIUS`, `DESPAWN_RADIUS`, `STREAM_INTERVAL_SECS`) is defined as compile-time constants in `core/src/constant.rs`; there is no `ChunkConfigValue` resource or `chunk.ron` asset.

## Simulation-set contract

The simulation-set ordering contract is stated here; set ordering is configured per schedule in `main.rs`.

| Schedule | Set | Contents |
|---|---|---|
| `PreUpdate` | `Input` | keyboard/mouse, UI buttons, pause toggle, editor input |
| `FixedFirst` | `Stats` | `PowerSet::{Generate, Boost, Demand, Solve}` (chained) |
| `FixedPreUpdate` | `Stats` | thrust/rotate stats (`StatsUpdateSet`), then `ship::update_stats` |
| `FixedUpdate` | `Ai` | AI think/steering |
| `FixedUpdate` | `Control` | player forces, camera follow target, drop collection |
| `FixedUpdate` | `Weapons` | turret targeting/tracking, shooting, projectiles, radar, ammo |
| `FixedUpdate` | `Cleanup` | lifetime countdown (`update_lifetime`), then chunk streaming (`stream_chunks` on `StreamingTimer`) |
| `FixedPostUpdate` | `Damage` | collisions → damage events → health, laser hit resolution |
| `FixedPostUpdate` | `Death` | block destruction → `BodyMutation` → debris/drops |
| `PostUpdate` | `Present` | camera, HUD, indicators, parallax |
| `Last` | `Cleanup` | deferred despawns |

`Damage → Death → BodyMutation` is chained inside one fixed tick: `Damage`
precedes `Death` via `configure_sets`, `apply_body_mutations` runs after
`handle_block_destroy`, and `handle_drop` runs after the mutation. Message
persistence across frames is never assumed.

## Schedules and ordering

| Schedule | Systems |
|---|---|
| `PreStartup` | `init_world_session` |
| `Startup` | `setup_camera`, `ConfigPlugin::load_config_handles`, `QuestPlugin::load_quest_handles` |
| `OnEnter(ConfigState::Ready)` | `load_default_fleet` |
| `PreUpdate` | input handling; Bevy state transitions |
| `FixedFirst` (`Stats`) | `PowerSet::{Generate, Boost, Demand, Solve}` chained: generation, booster priority & apply, active demand, proportional split & battery |
| `FixedPreUpdate` (`Stats`) | Thruster stat systems (`StatsUpdateSet`), then `ship::update_stats` (after `StatsUpdateSet`) |
| `FixedUpdate` (`Ai` → `Control` → `Weapons` → `Cleanup`) | AI chain, player control/camera, turrets/bullets/ammo, lifetime countdown, then `stream_chunks` (`.after(update_lifetime)`) |
| `FixedPostUpdate` (`Damage` → `Death`) | Damage events and laser hits; block destruction, mutations, drops, quest progression |
| `PostUpdate` (`Present`) | indicators, parallax, UI presentation |
| `Update` (`Input` → `Present`) | Editor/UI input, then config bridges, save/load requests, notices |

World loads are state transitions: `LoadSlotEvent` → `WorldContext::Restoring` →
`SceneState::Loading` → `SceneState::Game`. On `Game` entry,
`spawn_loaded_world` applies the snapshot and converts `Restoring` to `SaveSlot`;
skirmish spawning skips while `WorldContext::is_restoring()` is true.
`SaveWorldEvent` commits atomically on the auto-save timer and on
Pause `Back to Home` / `Ship Editor`.

`Time::<Fixed>::from_hz(60.0)`; Rapier runs in the fixed schedule with 2D gravity zero,
pixels-per-meter 1.0.

## Fixed-timestep physics contract

- Rapier `in_fixed_schedule()`, 60 Hz.
- `Time<Virtual>` is paused/resumed by `ui::modal::sync_modal_time` while `Modal::Pause` is the top modal.
- Ships are `RigidBody::Dynamic`, `ColliderMassProperties::Mass(ship.mass)`, `Sensor` block
  colliders with mass 0, ship voxel collider non-sensor.
- Bullets/missiles/hitboxes are `Sensor`s moving via direct `Transform` integration, not forces;
  ships are moved exclusively by `ExternalForce` (and `Velocity` for angular reset).

## Plugins

`main.rs` wires: `RapierPhysicsPlugin`, `HanabiPlugin`, diagnostics
(debug builds only), the scene plugins, plus `ConfigPlugin` (ron config boot gate),
`ControlPlugin`, `QuestPlugin`, `UiPlugin`, `WidgetPlugin`, `WorldPlugin`,
`ContentPlugin` (bullet, ship, block, item, lifetime, death, damage, selectable),
`SavePlugin` (`DesignSavePlugin` + `WorldSavePlugin`), and (feature `dev`)
`AssetCheckPlugin` / `WorldInspectorPlugin` / `RemotePlugin` / `BrpExtrasPlugin`.
With feature `dev`, `ContentPlugin` also registers `ShipDebugPlugin`, which draws
ship gizmos for the currently inspected ship; all of it is compiled out without the feature.

State gating on systems uses `run_if(in_state(SceneState::…))`, `on_timer`,
`on_message`, and `ConfigState`.

`bevy-inspector-egui` and `bevy_brp_extras` are optional dependencies, enabled only by
the `dev` cargo feature — production builds contain no inspector code.
`bevy_hanabi` is added but no effect assets exist yet.

## Module file roles

Every feature module is a folder with `mod.rs` as its root. The `Plugin`
implementation for a module is declared in that module's `mod.rs`, alongside
submodule declarations and `pub use` re-exports that keep existing import paths
stable when a plugin moves from a flat file into a folder.

| File | Responsibility |
|---|---|
| `mod.rs` | module root: submodule declarations, re-exports, plugin declaration |
| `system.rs` | systems and helper functions only; no enums, structs, traits, or plugins |
| `component.rs` | enums, structs, traits, and components owned by the module |
| `lib.rs` | helpers used by sibling modules |

Within each file, type declarations precede functions and the test module is
last. Crate paths are imported with top-of-file `use` declarations rather than
written inline in signatures, parameters, or bodies. `core/tests/conventions.rs`
enforces these invariants against `core/src/**/*.rs`.

## Module map (content)

```
content/
  team.rs        Team { id: u8 } + Rapier collision groups, 16 palette entries
  mod.rs         ContentPlugin — aggregate
  block/         BlockPlacement/GridBlock/BlockRef/BlockOwner, BlockRegistry, BlockDef/Shape/Voxel, features (turret, thruster, power, storage, ammo, sprite, shield), radar, stats (StackRule, StatEffect)
  body/          BlockGrid + BodyMutation; debris/ (Debris body, detach/split, expiry drops)
  ship/          Ship, AI controller, movement, geometry helpers, indicators, Stat computation, vision/, spawn/, throttle/
    inspect/     InspectedShip, click observer, Escape/despawn handling (enemy click gated to dev)
    debug/       (dev feature only) gizmo renderer driven by InspectedShip
  bullet/        Bullet/Laser/Missile components + all projectile systems
  damage/        Damage/DamageEvent (health decrement)
  death/         ShipDestroyed message
  lifetime/      Lifetime → despawn
  item/          ItemRegistry/ItemType/Rarity
  selectable/    drag-select via Ctrl+Left drag
quest/           QuestConfig/QuestCatalogDef/QuestTracker/HUD/procedural generator
config/          ConfigPlugin: ron loader, boot gate, GameConfig/BlocksConfig/GameSetting
save/
  design/        ShipCatalog, DesignCatalogSource, sandbox designs, fleet helpers
  world/         WorldStore (SQLite/in-memory), WorldContext, snapshots, save-slot lifecycle, auto-save
  document.rs    RON version checks and atomic writes for sandbox designs
```

## Lints

Workspace lints are configured in the root `Cargo.toml` (`[workspace.lints]`) and inherited by `core/Cargo.toml` via `[lints] workspace = true`:

- `rust::dead_code = "warn"`
- `rust::unused_variables = "warn"`
- `rust::unused_imports = "warn"`
- `clippy::absolute_paths = "deny"`
