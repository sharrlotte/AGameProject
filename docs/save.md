# Persistence Spec — Designs and World Slots (v2)

## Locations

`data_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`
(e.g. on Windows: `%APPDATA%\somerandomdev\agameproject\data`). Sub-directories:

```
<data_dir>/designs/*.ron        one self-contained ship design per file
<data_dir>/saves/battle.ron     single active campaign battle snapshot
<data_dir>/saves/campaign.ron   campaign/mission progression document
```

Old `schematics/*.json`, `schematics/*.ron`, `saves/slot_*.ron`, and `world/` are ignored
and untouched. There is no JSON reader, no v1/v2 slot reader, no phantom-ship
fabrication, and no "repair ships with <= 2 blocks".

## Design format (editor)

`designs/<name>.ron`, written by `ShipCatalog::save`:

```ron
(
  version: 1,
  id: "<uuid-v7>",
  name: "My Ship",
  blocks: [
    (id: "<uuid>", position: (0, 0), block_type: "core", rotation: 0),
  ],
  ai: Default,
)
```

`position` is grid-space (`IVec2`); `block_type` is a plain `String` id. The file
stem is derived from `name` by `ShipCatalog::file_name`. On first run, bundled
starter designs (`interceptor.ron`, `raider.ron`) are copied into `designs/`.
Missions and the editor both consume `ShipDesign` directly; there is no
separate schematic struct or `load_saved_schematics()`.

## Document API

`save/document.rs` owns every versioned RON read/write path: `RonDocument`
(`VERSION`, `KIND`), `Versioned<T>`, `parse_document`, `read_document`,
`write_document`, and `atomic_write`. Writes go to `.tmp`; the previous file is
copied to `.bak`, then the temporary file is renamed over the target. Unknown
versions are rejected before gameplay data is trusted. `ron_document!` implements the trait for `WorldSave` (version 3), `CampaignSave`
(version 1), and `ShipDesign` (version 1).

## World battle format (`SAVE_SCHEMA_VERSION = 3`)

`SaveWorldEvent` writes `saves/battle.ron` when `BattleMode::Campaign` is active.
`BattleMode::QuickBattle` never reads, writes, or clears `battle.ron` or `campaign.ron`.
The save is self-contained; all ships are embedded. There are **no per-ship files**,
no `slot`, and no `ship_ids`.

```ron
(
  version: 3,
  saved_at_unix_ms: 1700000000000,
  battle: (mission: "quick-battle", elapsed_secs: 42.5, outcome: Some("Victory")),
  camera: (x: 12.0, y: -4.0, zoom: 3.0),
  ships: [
    (
      id: "<uuid>", team: 0,
      transform: (x: 0.0, y: 0.0, rotation: 0.0),
      velocity: (x: 0.0, y: 0.0, angvel: 0.0),
      ai: Aggressive,
      ai_runtime: (behavior: "Idle", timer_secs: 0.25),
      blocks: [
        (
          placement: (id: "<uuid>", position: (0, 0), block_type: "core", rotation: 0),
          health: 5000.0, max_health: 5000.0,
          turret: None, ammo: None,
          single_pattern: None, burst_pattern: None,
          laser_pattern: None, pulling_pattern: false,
        ),
      ],
    ),
  ],
  drops: [
    (position: (10.0, 2.0), item_type: "steel", amount: 10, remaining_secs: 30.0),
  ],
  rng: (seed: 1234),
)
```

Everything that affects a continuing battle is captured: ship transforms and
Rapier velocities, block health/ammo/turret/pattern state, AI profile and
runtime behavior, battle timer and outcome, drops with remaining lifetime,
camera, and the seeded `GameRng`.

Never serialized: projectiles, lasers, pulling beams, damage text, and transient
VFX. They are tagged `DespawnOnExit(SceneState::Game)`, so exiting the old scene
clears them before the snapshot is spawned. Turret targets are also omitted and
reacquired after load.

`capture`/`restore` live in `save/snapshot.rs` behind the `CaptureSnapshot` and
`RestoreSnapshot` traits. Entity references are never persisted.

## Auto-save, Load & Reset flow

```
Home → Continue (when battle.ron exists)
  → LoadWorldEvent → PendingLoad { active: true }
  → NextState<SceneState>::Loading
  → exit outgoing scene → enter Game (spawn_loaded_world restores battle.ron)
  → setup_battle skips while PendingLoad is set
```

Auto-save triggers in `BattleMode::Campaign`:
- Periodic timer during `BattlePhase::Active` + `PauseState::Playing` (`auto_save_interval_secs` in `game.ron`).
- On `Pause` **Back to Home** (`PendingExitAction::Home` flushes after `SaveWorldEvent` completes).
- On `BattlePhase::Ended`: `battle.ron` is cleared; `Victory` records the mission in `campaign.ron`.

Main Menu **Settings → Data → Reset Mission / Campaign** (2-step confirmation) deletes
`battle.ron` and `campaign.ron` (and `.bak` files) while keeping `designs/*.ron` intact.
Load failures post an `AppNotice` and return to Home; no partial world is left behind.
