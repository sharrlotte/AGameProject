## REMOVED Requirements

### Requirement: A per-ship cell grid is the source of truth for block occupancy

**Reason**: Retained and strengthened in `ship-grid`; the separate cache capability is replaced by the canonical grid.

**Migration**: Use `ShipGrid.cells` and `BlockRegistry::stats_from_grid`.

### Requirement: Cache is maintained incrementally on spawn and despawn

**Reason**: Mutations now update `ShipGrid` and recompute derived stats, bounds, and collider through one pipeline.

**Migration**: Route all changes through `apply_ship_mutations`.

### Requirement: Fallback to full rebuild is available and documented

**Reason**: `rebuild_from_layout` is the spawn and editor path, and `ShipGrid` is authoritative.

**Migration**: Use the `ship-grid` rebuild and validation requirements.
