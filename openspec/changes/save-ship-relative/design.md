## Context

Current format (`save/io.rs`):
`test.json` — a single blob with: ships (id → ShipState/Team), blocks (id →
parent id → BlockData), turrets (id → parent → TurretState), transforms
(parent-uuid → Transform), healths, single shoot states.
Round-trip requires two passes: parse, spawn everything through `Commands`,
then re-link by uuid→Entity. Any mistake in that linking is silent
(e.g. existing known leaks: missiles/lasers/drops survive Load by virtue of
not using `SceneEntity(Game)`/`OnExit` cleanup — but the leak is precisely
because the schema cannot map them to a ship).

## Goals / Non-Goals

**Goals:**
- Ship-relative references everywhere: a ship's block data is
  `(position, block_type, rotation, health, pattern state, team)`, all inside
  one object — no UUIDs at rest.
- Separate files: `ships/<id>.ron` per ship; `world.ron` for world stamp
  (RNG seed, teams, times, drops, camera position).
- Versioned schema: `read_version()` strict — unknown versions error out
  loudly.
- Delete the uuid→Entity two-pass linker; spawn directly from relative data.
- Scene cleanup on load: everything that belonged to a ship is respawned
  from its ship file; everything not (missiles, lasers, drops, effects)
  despawned via scene-exit cleanup.

**Non-Goals:**
- Schematic `.ron` format for the *editor* — separate concern; the same
  serializer infrastructure can be reused but editor save stays compatible
  with existing `.json` schematics in this slice, to keep diff focused.
- World save of drops/lasers in-flight — no.
- Migration tooling for old JSON blobs — fresh format, break is acceptable.

## Decisions

- **`ships/<id>.ron` + `world.ron` two-file split.** Rejected: single file
  with all ships — same linking problem, just RON-flavored; simpler to debug
  per file.
- **Relative references use `(ship_id, block_index)` encoded as path in
  file** (`ships/ship-123-… .ron`) + `position: (i32, i32)` in-block instead
  of uuids. RNG id for ship chosen at save time; trust on load.
- **Schema version constant `WORLD_SCHEMA_VERSION = 1`** written at the top of
  both files; strict verify at read — unknown rejected.
- **Team preserved; Autoturret state preserved; health + max_health preserved;
  pattern state (single/burst) preserved with the same struct shapes.**
  Turrets addressed by relative index. Refactor Turrets/Health/PatternState
  to serialize inside `ShipSave` instead of beside it.
- **Drop items/lasers/missiles/effects are NOT saved** — they do not exist in
  the schema and therefore cannot leak. Documented explicitly.

## Risks / Trade-offs

- [Risk] Rethink all spawn sites to go through one `spawn_ship_from_save`
  entry — drift if any site hand-rolls its own spawn. → Mitigation: one entry
  point (`save::load_ship`), reused by pause-menu load and initial spawn.
- [Risk] Renaming a ship (editor rename) now means renaming the file — stale
  reference if editor saves by name. → Editor save stays compatible (file per
  save slot named by id, only the Displayed name changes).
- [Risk] Version constant mistakes are catastrophic (saves silently fail).
  → `read_version()` is strict and the loader panics with a clear message;
  developer-friendly for 0.1.0.

## Migration Plan

1. Design `ShipSave` struct + `WorldSave` struct; implement read/write via
   `ron`.
2. Rewrite pause-menu Save/Load to iterate per ship.
3. Delete the two-pass linker; document behaviour in `docs/save.md`.
4. Smoke test: save from pause → quit to Home → Load → verify ship state.

Rollback: revert — no schema migration required by users at 0.1.0.
