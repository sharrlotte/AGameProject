# Architecture

## Workspace layout

```
AGameProject/                Cargo workspace (resolver = "3")
  core/                      single crate, bin target (host application)
    assets/                  assets (root of asset lookup; see assets.md)
    src/
      main.rs                plugin wiring, window, audio, fixed-timestep setup
      resource.rs            states + typed shared resources
      simulation.rs          named simulation-set ordering contract
      system/                camera, startup ship load, debug scene audit
        mod.rs               module root, plugin declaration, debug gate
      constant.rs            global constants (incl. AI_THINK_RATE_HZ)
      content/               gameplay content (one module per domain)
      scene/                 UI/state scenes
        editor/              editor module root (state, plugin wiring) + feature modules
          mod.rs             module root: declarations, re-exports, plugin
          {ui,input,grid,save,ship_list,ai_panel}.rs  editor feature modules
        {background,game,home,notice,pause,setting,widget}/  scene modules
          mod.rs             module root: declarations, re-exports, plugin
        layout.rs            shared UI layout constants
      mission/               battle-core scaffolding (quick-battle.ron, overlay)
      config/                ron config boot gate
      save/                  serialization (`document.rs`, `io.rs`, `schema.rs`, `catalog.rs`, `snapshot.rs`)
```

`release` profile: `lto = "thin"`, `codegen-units = 1`, `opt-level = "s"`, stripped.
Dev profile optimizes dependencies, not the workspace crate.

## States

Five state machines, all Bevy 0.20 states:

| State | Source | Variants | Default |
|---|---|---|---|
| `SceneState` | `resource.rs` | `Loading`, `Home`, `Game`, `ShipEditor` | `Loading` |
| `PauseState` | `resource.rs` | `Playing`, `Paused` | `Playing` |
| `SettingState` | `resource.rs` | `Closed`, `Open` | `Closed` |
| `BattlePhase` | `mission/state.rs` | `Idle`, `Loading`, `Active`, `Ended` | `Idle` |
| `ConfigState` | `config/mod.rs` | `Loading`, `Ready`, `Error(ConfigError)` | `Loading` |

`SettingState` is registered inside `SettingScenePlugin`, the others in `main.rs`.
Scene-scoped entities are tagged with Bevy's `DespawnOnExit(S)`, one per owning
state: `SceneState::Game/Home/ShipEditor`, `PauseState::Paused`, and
`SettingState::Open`. Bevy despawns the tagged root and all of its children on
the matching state exit; no manual cleanup loop exists. A debug-only audit in
`system/mod.rs` samples root entities every five seconds and warns about any that
lack a `DespawnOnExit` tag.

Config load failures move `ConfigState` to `Error(ConfigError)` and show a
full-screen error instead of panicking. Mission load failures post an
`AppNotice` and return to `SceneState::Home`. All persisted data is RON;
`save/document.rs` owns version checks and atomic `.tmp` → `.bak` writes.

## Typed shared resources

| Resource | Contents |
|---|---|
| `DeployPayload` | `current_ship: Option<ShipDesign>` — Deploy payload consumed by mission spawn |
| `CameraTarget` | `following: Option<Entity>` — camera-follow ship |
| `ControlledShip` | `controlling: Option<Entity>` — ship receiving player input |
| `ControlMode` | `Pilot` (WASD flies the controlled ship, camera follows) or `FreeLook` (WASD pans, thrust paused); `F` toggles, middle-mouse drags the camera in either mode |
| `PendingLoad` | `active: bool` — set by `handle_load_request`, consumed on `SceneState::Game` entry |
| `GameConfigValue` | `GameConfig` from `game.ron` (damping, thrust, AI tuning, drops) |

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
| `FixedUpdate` | `Cleanup` | lifetime countdown |
| `FixedPostUpdate` | `Damage` | collisions → damage events → health, laser hit resolution |
| `FixedPostUpdate` | `Death` | block destruction → `BodyMutation` → debris/drops |
| `PostUpdate` | `Present` | camera, HUD, indicators, damage text, parallax |
| `Last` | `Cleanup` | deferred despawns |

`Damage → Death → BodyMutation` is chained inside one fixed tick: `Damage`
precedes `Death` via `configure_sets`, `apply_body_mutations` runs after
`handle_block_destroy`, and `handle_drop` runs after the mutation. Message
persistence across frames is never assumed.

## Schedules and ordering

| Schedule | Systems |
|---|---|
| `PreStartup` | `register_default_items` |
| `Startup` | `setup_camera`, `load_current_ship`, `AmmoPlugin::setup_assets`, `ConfigPlugin::load_config_handles` |
| `PreUpdate` | input handling; Bevy state transitions |
| `FixedFirst` (`Stats`) | `PowerSet::{Generate, Boost, Demand, Solve}` chained: generation, booster priority & apply, active demand, proportional split & battery |
| `FixedPreUpdate` (`Stats`) | Thruster stat systems (`StatsUpdateSet`), then `ship::update_stats` (after `StatsUpdateSet`) |
| `FixedUpdate` (`Ai` → `Control` → `Weapons` → `Cleanup`) | AI chain, player control/camera, turrets/bullets/ammo, lifetime countdown |
| `FixedPostUpdate` (`Damage` → `Death`) | Damage events and laser hits; block destruction, mutations, drops |
| `FixedLast` (`Cleanup`) | `death::despawn_dead` |
| `PostUpdate` (`Present`) | `update_ship_behavior`, indicators, parallax, UI presentation |
| `Update` (`Input` → `Present`) | Editor/UI input, then config bridges, save/load requests, mission polling, notices |

World loads are state transitions: `LoadWorldEvent` → `PendingLoad` →
`SceneState::Loading` → `SceneState::Game`. On `Game` entry,
`spawn_loaded_world` spawns the snapshot instead of `setup_battle` (which skips
while a load is pending).

`Time::<Fixed>::from_hz(60.0)`; Rapier runs in the fixed schedule with 2D gravity zero,
pixels-per-meter 1.0.

## Fixed-timestep physics contract

- Rapier `in_fixed_schedule()`, 60 Hz.
- `Time<Virtual>` is paused/resumed by the pause scene (`OnEnter`/`OnExit` `PauseState::Paused`).
- Ships are `RigidBody::Dynamic`, `ColliderMassProperties::Mass(ship.mass)`, `Sensor` block
  colliders with mass 0, ship voxel collider non-sensor.
- Bullets/missiles/hitboxes are `Sensor`s moving via direct `Transform` integration, not forces;
  ships are moved exclusively by `ExternalForce` (and `Velocity` for angular reset).

## Plugins

`main.rs` wires: `RapierPhysicsPlugin`, `HanabiPlugin`, diagnostics
(debug builds only), the scene plugins, plus `ConfigPlugin` (ron config boot gate),
`ContentPlugin` (bullet, ship, block, item, lifetime, death, damage, selectable),
`MissionPlugin` (battle-core scaffolding), `SavePlugin`, and (debug builds / feature
`dev`) `WorldInspectorPlugin` / `RemotePlugin`. With feature `dev`, `ContentPlugin` also
registers `ShipDebugPlugin`, which owns click selection, the scoped debug label, and
ship gizmos; all of it is compiled out without the feature.

State gating on systems uses `run_if(in_state(SceneState::…))`, `on_timer`,
`on_message`, and explicit state `States` (`BattlePhase`, `ConfigState`).

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
  block/         BlockPlacement/GridBlock/BlockRef/BlockOwner, BlockRegistry, BlockDef/Shape/Voxel, features (turret, thruster, power, storage, ammo, sprite), radar, stats (StackRule, StatEffect)
  body/          BlockGrid + BodyMutation; debris/ (Debris body, detach/split, expiry drops)
  ship/          Ship, AI controller, movement, geometry helpers, indicators, Stat computation
    debug/       (dev feature only) ShipDebugSelection, click observer, scoped text, gizmo renderer
  bullet/        Bullet/Laser/Missile components + all projectile systems
  damage/        Damage/DamageEvent (health decrement + floating text)
  death/         Death marker → despawn
  lifetime/      Lifetime → Death
  item/          ItemRegistry/ItemType/Rarity
  selectable/    drag-select via Ctrl+Left drag
mission/         MissionDef .ron, BattlePhase state, battle spawn/outcome/overlay
config/          ConfigPlugin: ron loader, boot gate, GameConfig/BlocksConfig/GameSetting
```

## Lints

Workspace lints are configured in the root `Cargo.toml` (`[workspace.lints]`) and inherited by `core/Cargo.toml` via `[lints] workspace = true`:

- `rust::dead_code = "deny"`
- `rust::unused_variables = "deny"`
- `rust::unused_imports = "deny"`
- `clippy::absolute_paths = "deny"`
