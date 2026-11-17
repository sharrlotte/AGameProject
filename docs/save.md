# Persistence Spec — Schematics and World Saves

## Locations

`save_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`
(e.g. on Windows: `%APPDATA%\somerandomdev\agameproject\data`).
Sub-directories: `schematics/` (ship blueprints, JSON) and `world/` (RON world saves).

## Schematic format (editor, unchanged)

`<filename>.json`:

```json
{
  "id": "<uuid-v7>",
  "filename": "my-ship",
  "name": "My Ship",
  "blocks": [
    { "id": "<uuid>", "position": { "x": 0, "y": 0 }, "block_type": "core", "rotation": 0 }
  ]
}
```

`position` is grid-space (`IVec2`). `block_type` is an interned `Symbol` id string.
Editor schematic save/load stays JSON (Slice 6 keeps this format focused on world saves).

On load, `load_saved_schematics()`:
1. Reads `schematics/`, parses `*.json` into `SchematicData`, overriding `filename`.
2. Pads up to 5 placeholder ships when fewer exist (name `ship_N`, one Core + one
   Laser Turret if a schematic has ≤ 2 blocks — in memory only).
3. `SchematicData::to_ship_data()` clones blocks and assigns **new UUIDs** to every block
   on spawn.

## World save format (Slice 6, ship-relative RON)

Pause → **Save** writes two kinds of file under `world/`:

```
world/
  world.ron
  ships/<ship-uuid>.ron   (one per ship)
```

`world.ron` — world-level stamp only:

```ron
(
    version: 1,
    teams: [0, 1],
    elapsed_secs: 12.5,
    ship_ids: ["<uuid>", ...],
)
```

`ships/<uuid>.ron` — one ship, blocks relative to the ship (no UUID cross-refs):

```ron
(
    version: 1,
    id: "<uuid>",
    team: 0,
    blocks: [
        (
            position: (0, 0),
            block_type: "core",
            rotation: 0,
            health: 5000.0,
            max_health: 5000.0,
            turret_state: None,
            single_pattern: None,
            burst_pattern: None,
        ),
        (
            position: (1, 3),
            block_type: "laser_turret",
            rotation: 0,
            health: 1600.0,
            max_health: 1600.0,
            turret_state: Some((active: false, shoot_at: None)),
            single_pattern: None,
            burst_pattern: Some((shooting: Cooldown(last_fired_at: 0.0))),
        ),
    ],
)
```

Notes:
- `WORLD_SCHEMA_VERSION = 1` is written at the top of both files. `read_version()`
  rejects unknown versions loudly, naming expected and found versions.
- Block positions are grid-relative to the ship; there are no UUIDs at rest inside a ship
  file, and no two-pass uuid→Entity linker on load.
- Turret state is saved without entity references (a loaded world has no acquired targets).
- Single/burst pattern states are preserved with their existing struct shapes.
- **Nothing else is persisted**: no missiles, lasers, drops, beams, lifetimes, damage text,
  camera, or selection state. On load all `Save`-marked effect entities are despawned, so a
  loaded world cannot leak them.
- Ship files are named by a UUID derived from the ship entity at save time (stable for the
  entity's lifetime); stale files from earlier saves are not deleted in 0.1.0.
- Old `saves/test.json` files do not load — no migration, fresh start expected.

## Save API

`SaveWorldEvent`, `LoadWorldEvent` — unit messages (no path); `handle_save_world_event` /
`handle_load_world_event` run in `Update`, `on_message` gated, in `SimulationSet::Apply`.
`spawn_ship_from_save` is the single entry point for recreating a ship from a `ShipSave`.
`save_schematic(&SchematicData)` remains callable directly by the editor.
