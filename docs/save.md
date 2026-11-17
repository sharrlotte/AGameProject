# Persistence Spec — Designs and World Slots (v2)

## Locations

`data_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`
(e.g. on Windows: `%APPDATA%\somerandomdev\agameproject\data`). Sub-directories:

```
<data_dir>/designs/*.ron        one self-contained ship design per file
<data_dir>/saves/slot_1.ron … slot_5.ron
```

Old `schematics/*.json`, `schematics/*.ron`, and `world/` are ignored and
untouched. There is no JSON reader, no v1 world reader, no phantom-ship
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
versions are rejected before gameplay data is trusted. `ron_document!`
implements the trait for `WorldSave` (version 2) and `ShipDesign` (version 1).

## World slot format (`SAVE_SCHEMA_VERSION = 2`)

`SaveWorldEvent { slot }` writes `saves/slot_N.ron`. Each slot is self-contained;
all ships are embedded. There are **no per-ship files** and no `ship_ids`.

```ron
(
  version: 2,
  slot: 1,
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

## Load flow

```
SaveWorldEvent/LoadWorldEvent { slot }
  → PendingLoad { slot }
  → NextState<SceneState>::Loading
  → exit Game (DespawnOnExit clears the old scene)
  → enter Game (spawn_loaded_world restores the embedded ships)
  → setup_battle skips while PendingLoad is set
```

The Home and Pause screens both show a five-slot panel (`scene/slot_panel.rs`)
with mission name, ship count, elapsed time, and a relative save timestamp.
Load failures post an `AppNotice` and return to Home; no partial world is left
behind.
