# Architecture

## Workspace layout

```
game/                        Cargo workspace (resolver = "3")
  core/                      single crate, bin target (host application)
    assets/                  assets (root of asset lookup; see assets.md)
    src/
      main.rs                plugin wiring, window, audio, fixed-timestep setup
      resource.rs            states + shared resources
      system.rs              camera, startup ship load, scene-entity cleanup
      constant.rs            global constants
      content/               gameplay content (one module per domain)
      scene/                 UI/state scenes
      save/                  serialization
      ui/                    reserved (empty)
      utils/                 helpers
      world/                 reserved (empty)
```

`release` profile: `lto = "thin"`, `codegen-units = 1`, `opt-level = "s"`, stripped.
Dev profile optimizes dependencies, not the workspace crate.

## States

Three state machines, all Bevy 0.20 states:

| State | Source | Variants | Default |
|---|---|---|---|
| `SceneState` | `resource.rs` | `Home`, `Game`, `ShipEditor` | `Home` |
| `PauseState` | `resource.rs` | `Playing`, `Paused` | `Playing` |
| `SettingState` | `resource.rs` | `Closed`, `Open` | `Closed` |

`SettingState` is registered inside `SettingScenePlugin`, the others in `main.rs`.
Scene-scoped entities are tagged `SceneEntity(SceneState)`; `clean_state_item`
(`system.rs`) despawns them on every `SceneState` exit transition (`PreUpdate`).
Entities without the tag survive scene transitions.

## Schedules and ordering

| Schedule | Systems |
|---|---|
| `PreStartup` | `register_default_items` |
| `Startup` | `setup_camera`, `load_current_ship`, `AmmoPlugin::setup_assets`, `ConfigPlugin::load_config_handles` |
| `PreUpdate` | `clean_state_item` |
| `FixedFirst` | Efficiency systems (chained, in `EfficiencyUpdateSet`), then energy systems (`.after(EfficiencyUpdateSet)`, chained) |
| `FixedPreUpdate` | Thruster stat systems (`StatsUpdateSet`), then `ship::update_stats` (after `StatsUpdateSet`) |
| `FixedUpdate` | Bullet/damage/lifetime/selectable-independent logic: turrets, bullets, missiles, single/burst fire, laser beam, ammo warnings, ships (AI, player control, collider rebuild, UI), drop collection, radar warning timers |
| `FixedPostUpdate` | Damage application, block damage flash overlay, drop spawn, block destroy, DamageEvent → health |
| `FixedLast` | `death::despawn_dead` (despawn entities with `Death`) |
| `PostUpdate` | `update_ship_behavior` |
| `Update` | Radar scans, selection, editor systems, HUD/UI update, FPS text, ship list rebuild |

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
  entity.rs      EntityType enum (Ship, Block, Bullet, Missile, Laser, Ammo)
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
