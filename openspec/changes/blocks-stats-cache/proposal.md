## Why

`BlockRegistry::blocks_stats` rebuilds a full cell map and flood fill from
the ship's entire block list on every `BlockDestroyedEvent`, which is quadratic
in the number of blocks for heavy combat bursts. The same data is needed by
collider rebuild (same event) and fragment spawning — already paid for but
never cached.

## What Changes

- Maintain a per-ship `ShipGrid` resource (already reflected today) as the
  authoritative cell grid for the live game; destroy/repair events update it
  incrementally instead of recomputing from scratch.
- `blocks_stats` takes a reference to the cached grid instead of rebuilding it
  inline; the BFS runs per event but is O(cells unaffected) in the stable case.
- `blocks_to_voxel_points` mirrors off the cached grid; collider rebuild uses
  the same structure.
- No API change to saved data (ship data still round-trips through the same
  block list).

## Capabilities

### New Capabilities
- `blocks-stats-cache`: per-ship block grid maintained incrementally;
  collider and fragment stats read from the cache.

### Modified Capabilities
<!-- No existing specs. -->

## Impact

- Code: `content/ship/components.rs` (ShipGrid lifecycle helpers),
  `content/ship/systems.rs` (`check_rebuild_ship_collider`),
  `content/block/components.rs` (`blocks_stats`, `blocks_to_voxel_points`),
  `save/io.rs` and editor spawn paths (grid construction on ship load),
  `docs/simulation.md`, `docs/ai.md`.
- Measured expectation: 5–10× fewer cell inserts per destroyed block in a
  50-block lobby case; exact multiplier depends on event burst profile.
