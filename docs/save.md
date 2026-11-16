# Persistence Spec — Schematics and World Saves

## Locations

`save_dir()` = `directories::ProjectDirs::from("com", "somerandomdev", "agameproject").data_dir()`
(e.g. on Windows: `%APPDATA%\somerandomdev\agameproject\data`) — created lazily on save.
Sub-directories: `schematics/` (ship blueprints) and `saves/` (world saves).

## Schematic format

`<filename>.json`:

```json
{
  "id": "<uuid-v7>",
  "filename": "my-ship",
  "name": "My Ship",
  "blocks": [
    { "id": "<uuid>", "position": { "x": 0, "y": 0 }, "block_type": "Core", "rotation": 0 }
  ]
}
```

`position` is grid-space (`IVec2`), anchored bottom-left. `rotation` is a `u8` flag byte
(0 = unrotated everywhere in practice). `block_type` is now an interned `Symbol` id
string (e.g. `"core"`, `"laser_turret"`, `"block_steel"`); legacy enum-name
schematics fail to load with an incompatibility error (Slice 2 breakage).

On load, `load_saved_schematics()`:
1. Reads `schematics/`, parses `*.json` into `SchematicData`, overriding `filename`.
2. Pads up to 5 placeholder ships when fewer exist (name `ship_N`, one Core + one
   LaserTurret added if a schematic has ≤ 2 blocks — in memory only).
3. `SchematicData::to_ship_data()` clones blocks and assigns **new UUIDs** to every block
   on spawn, so loaded schematics differ from saved ones.

Schematics are consumed by:
- Ship Editor (load/list/save/delete),
- `check_ship` (top-up spawning; reads the whole directory every 100 ms — see defects.md),
- `load_current_ship` (startup: picks `schematics[0]`).

## World save format

Pause → **Save** writes `saves/test.json`:

```json
{
  "ships":          [ { "id": "…", "state": {}, "team": { "id": 0, "color": … } } ],
  "blocks":         [ { "id": "…", "parent": "…", "block_type": "…", "position": {…}, "rotation": 0,
                        "state": { "block_type": "…" } } ],
  "turrets":        [ { "id": "…", "parent": "…", "state": { "active": false,
                        "target": null, "shoot_at": null } } ],
  "transforms":     [ { "parent": "…", "component": { … } } ],
  "healths":        [ { "parent": "…", "component": { "health": 5000.0, "max_health": 5000.0 } } ],
  "shoot_patterns": { "single": [ { "parent": "…",
                                   "component": { "last_fired_at": 0.0 } } ] }
}
```

Notes:
- Save walks **every** `Ship`, `Block`, `TurretState`, `Transform`, `Health`, `SingleShootPatternState`
  in the world. `get_parent` panics on entities without a parent (e.g. scene-independent
  ships).
- `Transform` is saved verbatim via `bevy`'s `Serialize` impl and restored verbatim.
- Load path: pause menu → despawn all ships/bullets → `LoadWorldEvent` → `handle_load_world_event`
  spawns blocks at recorded positions, re-inserts turret state, then **overrides Transform,
  Health and single-shot timing** through `ParentWrapper` reconstruction.
- Restored ships get fresh `AIShipController` (Idle), fresh colliders, default
  `ComputedShipStats`; saved `Transform` is applied after spawn.
- Nothing else is persisted: **no drops, lasers, missiles, beams, lifetimes — and no
  player camera/selection state.**
- `Load` after `Save` is not idempotent in the presence of game-notifications — see defects.md.

## Save API

`SaveWorldEvent { path: PathBuf }`, `LoadWorldEvent { path: PathBuf }` — messages;
`handle_save_world_event` / `handle_load_world_event` run in `Update`,
`on_message` gated.
`save_schematic(&SchematicData)` is also callable directly by the editor.
