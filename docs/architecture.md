# Architecture

## Workspace layout

```
game/                        Cargo workspace (resolver = "3")
  core/                      single crate, bin target (host application)
    assets/                  assets (root of asset lookup; see assets.md)
    src/
      main.rs                plugin wiring, window, audio, fixed-timestep setup
      resource.rs            states + typed shared resources
      simulation.rs          named simulation-set ordering contract
      system.rs              camera, startup ship load, scene-entity cleanup
      constant.rs            global constants
      content/               gameplay content (one module per domain)
      scene/                 UI/state scenes
      mission/               battle-core scaffolding (mission.ron, overlay)
      config/                ron config boot gate
      save/                  serialization (`document.rs` versioned RON IO, `io.rs`, `schema.rs`)
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
| `BattlePhase` | `mission/mod.rs` | `Loading`, `Active`, `Ended` | `Loading` |
| `ConfigState` | `config/mod.rs` | `Loading`, `Ready`, `Error(ConfigError)` | `Loading` |

`SettingState` is registered inside `SettingScenePlugin`, the others in `main.rs`.
Scene-scoped entities are tagged `SceneEntity(SceneState)`; `clean_state_item`
(`system.rs`) despawns them on every `SceneState` exit transition (`Cleanup`).
Entities without the tag survive scene transitions.

Config load failures move `ConfigState` to `Error(ConfigError)` and show a
full-screen error instead of panicking. Mission load failures post an
`AppNotice` and return to `SceneState::Home`. All persisted data is RON;
`save/document.rs` owns version checks and atomic `.tmp` → `.bak` writes.

## Typed shared resources

| Resource | Contents |
|---|---|
| `DeployPayload` | `current_ship: Option<SchematicData>` — Deploy payload consumed by mission spawn |
| `CameraTarget` | `following: Option<Entity>` — camera-follow ship |
| `ControlledShip` | `controlling: Option<Entity>` — ship receiving player input |
| `GameConfigValue` | `GameConfig` from `game.ron` (damping, thrust, AI tuning, drops) |

## Simulation-set contract

The target schedule contract (implemented incrementally; see
`openspec/changes/rewrite-game-systems/design.md` §3 and the doc comment on
`SimulationSet`):

| Schedule | Set | Contents |
|---|---|---|
| `PreUpdate` | `Input` | keyboard/mouse, UI buttons, pause toggle, editor input |
| `FixedPreUpdate` | `Stats` | power, efficiency, thrust/rotate stats, cargo aggregation |
| `FixedUpdate` | `Ai` | AI think/steering at `ai_think_rate_hz` |
| `FixedUpdate` | `Control` | player forces, camera follow target |
| `FixedUpdate` | `Weapons` | turret targeting/tracking, shooting, projectile movement, radar |
| `FixedUpdate` | `Physics` | Rapier step (engine) |
| `FixedPostUpdate` | `Damage` | collision events → damage events → health |
| `FixedPostUpdate` | `Death` | block destruction → `ShipMutation` → drops |
| `PostUpdate` | `Present` | camera, HUD, indicators, damage text |
| `Last` | `Cleanup` | deferred despawns |

`Damage → Death → ShipMutation` is chained inside the same fixed tick; message
persistence across frames is never assumed.

Until the split lands, `simulation.rs` exports `Input`, `Simulate`, `Apply`,
`Cleanup`; every ordering-sensitive system belongs to exactly one set.

| Set | Meaning |
|---|---|
| `Input` | poll devices/UI; fire on user actions (selection, editor placement, pause, zoom) |
| `Simulate` | think, integrate, collide, derive state (AI, turrets, radar, lifetimes, camera, stats) |
| `Apply` | turn events into lasting state (damage, drops, fragmentation, deaths, scene save) |
| `Cleanup` | despawn-dead, scene teardown hygiene |

Bevy's schedules run in order (`FixedFirst → FixedPreUpdate → FixedUpdate
→ FixedPostUpdate → FixedLast`; `PreUpdate → Update → PostUpdate`); sets
documented inside each plugin identify members of the pipeline
(`configure_sets(FixedFirst, SimulationSet::Simulate)`, etc.).

## Schedules and ordering

| Schedule | Systems |
|---|---|
| `PreStartup` | `register_default_items` |
| `Startup` | `setup_camera`, `load_current_ship`, `AmmoPlugin::setup_assets`, `ConfigPlugin::load_config_handles` |
| `PreUpdate` (Cleanup) | `clean_state_item` |
| `FixedFirst` (Simulate) | Efficiency systems (chained, `EfficiencyUpdateSet`), then energy systems (`.after(EfficiencyUpdateSet)`, chained) |
| `FixedPreUpdate` (Simulate) | Thruster stat systems (`StatsUpdateSet`), then `ship::update_stats` (after `StatsUpdateSet`) |
| `FixedUpdate` (mixed) | Simulate: turrets, bullets, missiles, fire/laser/pulling beam, ammo warnings, drops; Apply: `update_lifetime` (turns `Lifetime` into `Death`) |
| `FixedPostUpdate` (Apply) | Damage application, block damage flash overlay, block destroy, drop spawn |
| `FixedLast` (Cleanup) | `death::despawn_dead` |
| `PostUpdate` (Simulate) | `update_ship_behavior` |
| `Update` (Input/Simulate) | Handles/scroll/FPS (Input); radar scan, selection-gated simulations, outcome polls, list rebuilds (Simulate unless noted) |

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
`dev`) `WorldInspectorPlugin` / `RemotePlugin`.

State gating on systems uses `run_if(in_state(SceneState::…))`, `on_timer`,
`on_message`, and explicit state `States` (`BattlePhase`, `ConfigState`).

`bevy-inspector-egui` and `bevy_brp_extras` are optional dependencies, enabled only by
the `dev` cargo feature — production builds contain no inspector code.
`bevy_hanabi` is added but no effect assets exist yet.

## Module map (content)

```
content/
  team.rs        Team { id: u8 } + Rapier collision groups, 16 palette entries
  plugin.rs      ContentPlugin — aggregate
  block/         Block, BlockRegistry, BlockData/Def/Shape/Voxel, features (turret, thruster, energy, efficiency, storage, ammo, sprite), radar, stats (StackRule, StatEffect)
  ship/          Ship, AI controller, movement, UI, indicators, Stat computation
  bullet/        Bullet/Laser/Missile components + all projectile systems
  damage/        Damage/DamageEvent (health decrement + floating text)
  death/         Death marker → despawn
  lifetime/      Lifetime → Death
  item/          ItemRegistry/ItemType/Rarity
  selectable/    drag-select via Ctrl+Left drag
mission/         MissionDef .ron, BattlePhase state, battle spawn/outcome/overlay
config/          ConfigPlugin: ron loader, boot gate, GameConfig/BlocksConfig/GameSetting
```
