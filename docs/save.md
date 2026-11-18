# Persistence Spec — Designs, Campaign, and World Save (v5 / design v2)

## Module Layout

`core/src/save/` is organized by domain following the `mod.rs`, `component.rs`, `system.rs`, `lib.rs` structure:

```
core/src/save/
├── mod.rs          SavePlugin (registers DesignSavePlugin, CampaignSavePlugin, BattleSavePlugin)
├── document.rs     RonDocument trait, DocumentError, parse/read/write_document, atomic_write, data_dir
├── design/         DesignSavePlugin, ShipCatalog (Resource), starter seeding, design file I/O
├── campaign/       CampaignSavePlugin, CampaignSave (Resource), campaign progression & reset
└── battle/         BattleSavePlugin, BattleSaveState (Resource), WorldSave (v5), capture & restore
```

## Locations

`data_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`
(e.g. on Windows: `%APPDATA%\somerandomdev\agameproject\data`). Sub-directories:

```
<data_dir>/designs/*.ron        one self-contained ship design per file
<data_dir>/saves/battle.ron     single active campaign battle snapshot
<data_dir>/saves/campaign.ron   campaign/mission progression document
```

User settings (`config/setting.ron`) remain in `assets/config/setting.ron` for Bevy `AssetServer` hot-reloading.
Old `schematics/*.json`, `schematics/*.ron`, `saves/slot_*.ron`, and `world/` are ignored and untouched.

## Resource-Backed Caching

All persistent save domains are cached in ECS memory as Bevy `Resource`s initialized in `PreStartup`:
- `ShipCatalog` (`save::design::ShipCatalog`): seeded and loaded once at startup; updated in memory and on disk via `ShipCatalog::save_design`. Consumed by `load_current_ship`, `MissionPlugin`, and `ShipEditorPlugin` without repeated disk reads.
- `CampaignSave` (`save::campaign::CampaignSave`): loaded once at startup; updated via `CampaignSave::record_victory` and `CampaignSave::reset`.
- `BattleSaveState` (`save::battle::BattleSaveState`): tracks `has_save: bool` in memory so `Home` and `Settings` never poll the filesystem on UI updates.

## Design Format (`DESIGN_SCHEMA_VERSION = 2`)

`designs/<name>.ron`, written by `ShipCatalog::save_design`. `BlockPlacement.rotation` is
a `Rot90` (`R0`, `R90`, `R180`, `R270`) serialized by variant name. v1 designs are rejected
by the version check and left untouched on disk.

```ron
(
  version: 2,
  id: "<uuid-v7>",
  name: "My Ship",
  blocks: [
    (id: "<uuid>", position: (0, 0), block_type: "core", rotation: R0),
  ],
  ai: (
    initial: Idle,
    rules: [],
  ),
)
```

When a `ShipDesign` is spawned into the world (`ShipDesign::spawn` or `ShipDesign::spawn_with_id`), `Ship.id` holds a unique ship instance `Uuid` (preserved across block mutations and save/load), while `Ship.core_id` holds the core block's `BlockPlacement.id` in `BlockGrid`.

## Document API

`save/document.rs` owns every versioned RON read/write path: `RonDocument` (`VERSION`, `KIND`), `parse_document`, `read_document`, `write_document`, and `atomic_write`. Writes go to `.tmp`; the previous file is copied to `.bak`, then the temporary file is renamed over the target. Unknown versions are rejected before gameplay data is trusted. `ron_document!` implements the trait for `WorldSave` (version 6), `CampaignSave` (version 1), and `ShipDesign` (version 2).

## World Battle Format (`SAVE_SCHEMA_VERSION = 6`)

`SaveWorldEvent` writes `saves/battle.ron` when `BattleMode::Campaign` is active. `BattleMode::QuickBattle` never reads, writes, or clears `battle.ron` or `campaign.ron`.

```ron
(
  version: 6,
  saved_at_unix_ms: 1700000000000,
  battle: (mission: "quick-battle", elapsed_secs: 42.5, outcome: None),
  camera: (x: 12.0, y: -4.0, zoom: 3.0, following: Some("<ship-uuid>")),
  control: (controlling: Some("<ship-uuid>"), pilot_mode: true),
  bodies: [
    Ship((
      id: "<ship-instance-uuid>",
      team: 0,
      transform: (x: 0.0, y: 0.0, rotation: 0.0),
      velocity: (x: 0.0, y: 0.0, angvel: 0.0),
      cargo: (used: 10, items: {"steel": 10}),
      ai: (initial: Idle, rules: []),
      ai_runtime: (
        current: Attack(target: Some("<target-ship-uuid>")),
        spawn_position: (0.0, 0.0),
        rule_cooldowns: [0.0],
        timer_secs: 0.25,
      ),
      blocks: [
        (
          placement: (id: "<block-uuid>", position: (0, 0), block_type: "core", rotation: R0),
          health: 5000.0,
          max_health: 5000.0,
          battery_charge: Some(100.0),
          turret: Some((
            active: true,
            shoot_at: Some((100.0, 50.0)),
            barrel_rotation: 0.5,
            charge: Some(20.0),
          )),
          ammo: None,
          single_pattern: Some((cooldown_remaining: 0.25)),
          burst_pattern: None,
          laser_pattern: None,
          pulling_pattern: false,
        ),
      ],
    )),
    Debris((
      team: 1,
      transform: (x: 240.0, y: -64.0, rotation: 0.25),
      velocity: (x: 3.0, y: 0.0, angvel: 0.5),
      remaining_secs: 42.0,
      blocks: [
        (
          placement: (id: "<block-uuid>", position: (0, 0), block_type: "plate", rotation: R0),
          health: 40.0,
          max_health: 50.0,
          battery_charge: None,
          turret: None,
          ammo: None,
          single_pattern: None,
          burst_pattern: None,
          laser_pattern: None,
          pulling_pattern: false,
        ),
      ],
    )),
  ],
  drops: [
    (position: (10.0, 2.0), item_type: "steel", amount: 10, remaining_secs: 30.0),
  ],
  rng: (seed: 1234),
)
```

### Capture & Restore Architecture

- **Direct Hierarchical Capture ($O(1)$ Block Lookup)**: `WorldSave::capture` → `BodySnapshot::capture` → `ShipSnapshot`/`DebrisSnapshot` → `BlockSnapshot::capture(world, grid_block)` reads directly from `&GridBlock` without scanning `world.iter_entities()`.
- **Two-Pass Restore**:
  1. **Pass 1**: Spawns all ships with their saved instance `Ship.id`, restores velocity, `ShipStats` cargo (`cargo_used`, `cargo_items`), and block state (`Health`, battery `Charge`, `TurretState`, `TurretCharge`, barrel rotation on `TurretBarrel`, `AmmoState`, and session-independent relative weapon pattern timers), and builds `ship_map: HashMap<Uuid, Entity>`. Debris bodies are then spawned outside that map, with their saved transform, velocity, blocks, team, and remaining lifetime.
  2. **Pass 2**: Resolves `Option<Uuid>` references through `ship_map` to restore `AiRuntime` (`AiBehavior` target `Entity`, `spawn_position`, `rule_cooldowns`, `think_timer`), `CameraTarget`, `ControlledShip`, and `ControlMode`. Debris is never resolved as a camera, control, or AI target.
- **Drops & Battle Outcome**: Restored item drops include a visible `Sprite` (colored by `ItemRegistry` rarity and sized by `config.0.drop_size`). When `save.battle.outcome` is `Some(...)`, `spawn_loaded_world` restores `BattlePhase::Ended` so the outcome overlay is displayed.
