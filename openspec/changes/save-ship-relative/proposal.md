## Why

The current world save is a single `test.json` in JSON with UUIDs everywhere
and a schema that balloons to 60 lines of struct definitions (`ShipSave`,
`BlockSave`, `TurretSave`, `HealthSave`, etc.), round-tripped through code
that must re-assign entity ids after load, plus a schema that breaks silently
if two saves are opened side by side. A ship-relative RON format with one
file per ship removes the cross-reference dance entirely, aligns save/load
with the on-disk `.ron` config format, and is the substrate for a campaign
later.

## What Changes

- **BREAKING**: the world save is no longer JSON `String`+`serde_json`; it is
  RON using stable entity references through per-ship id (UUIDs scoped to ship).
- New format `ships/<id>.ron`: each ship saves itself (blocks with relative
  positions, turret state, health, single/burst pattern states, team);
  `world.ron` holds world stamp data (RNG seed, teams, time, persistent
  drops). Replaces `test.json` load/save flow entirely.
- Save format is versioned; `read_version()` hard-fails on unknown versions.
- Pause-menu Save/Load rewires to per-ship files + `world.ron`. Schematic-
  mode (editor) save is unaffected in behaviour; only the internal format
  shifts to the same serializer infrastructure for consistency.
- The pre-existing save/load known leaks (missiles/lasers/drops surviving
  load) are closed — scene entity cleanup now also covers world-level actors.

## Capabilities

### New Capabilities
- `ship-relative-save`: per-ship RON save files plus a `world.ron`; versioned
  schema; entity id remapping removed in favour of relative references.

### Modified Capabilities
<!-- No existing specs. -->

## Impact

- Code: `save/io.rs` rewrite, `save/mod.rs` split into `save/io.rs` and
  `save/schema.rs`, `scene/pause.rs` (save/load buttons), `system.rs`
  (`load_current_ship`), `mission/` (none — unrelated), `docs/save.md`,
  `docs/rewrite.md`.
- Dependency direction: `ron` already a direct dependency. No new deps.
- **BREAKING**: old `test.json` files will not load; no migration provided.
