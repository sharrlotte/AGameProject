## Context

The current hot path (`check_rebuild_ship_collider`) calls
`BlockRegistry::blocks_stats` per event, which:
  1. allocates `cell_map: HashMap<IVec2, &BlockData>`,
  2. iterates every block again for the flood fill,
  3. allocates `ShipCalculatedStat`.

All of this is recomputable from the block list, but the block list does not
change shape (position/size/rotation) between events for 95% of ticks. Caching
turns O(N²) bad cases into O(k) touch points for the k cells near the damage.

## Goals / Non-Goals

**Goals:**
- Per-ship grid (`IVec2 → BlockEntity`) owned by the ship; block spawn / death
  updates grid incrementally.
- `blocks_stats` reads cached grid instead of rebuilding cell_map per event.
- Fragment decision runs unchanged (BFS on grid of changed positions only if
  viable; else fall back to full-grid BFS — measure first).

**Non-Goals:**
- Spatial partitioning (hash grid) for turret hit queries — separate change.
- Altering fragmentation thresholds or Cores fragmentation semantics.

## Decisions

- **Reuse the editor `ShipGrid` type, hoisted into `content/ship/`**. Avoid
  duplicating the grid struct; one `ShipGrid` used by editor and runtime.
- **Incremental update on Add/Remove of `Block`** via observer or dirty-flag:
  recompute cell_map incrementally for the affected cells; full rebuild only
  when ship spawns/repairs. Rationale: drift bugs are hard; cache correctness
  must be provable.
- **Two-path strategy**: if grid is dirty (many cells changed), do one full
  rebuild; if clean, use grid BFS. Choose by cell dirty count, not by time.
- **Do not cache `ShipCalculatedStat` output itself** — only the cell map and
  `cell_map → block` adjacency; stats are recomputed cheaply from the map.

## Risks / Trade-offs

- [Risk] Cache staleness after repair/merge of fragmented ships. → Mitigation:
  grid is authoritative for position ownership; validate by `Entity` alive check
  on every access, refresh on spawn/destroy.
- [Risk] Memory cost of a `HashMap<IVec2, Entity>` per ship — trivially small
  (~few hundred cells).
- [Risk] Cache + editor dual users of the same type diverge over time. →
  Single shared module (`ship/grid.rs`) used by both.

## Migration Plan

1. Introduce `ShipGrid` in runtime `ship/grid.rs`, swap `blocks_stats` cells
   source to use it.
2. Wire spawn/destroy to maintain the grid; keep full-rebuild fallback.
3. Measure: burst event of destroying 30 blocks on a 64-block ship; compare
   frame times before/after (optional, document results in commit message).
