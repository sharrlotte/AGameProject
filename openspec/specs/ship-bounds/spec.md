# ship-bounds Specification

## Purpose
Ship-local `BoundingBox` representation (`min`, `max`, `center`, `size`) and world-space corner/center projection shared across gizmos, geometry helpers, and marquee selection.

## Requirements
### Requirement: BoundingBox stores ship-local min, max, center, and size
`BoundingBox` SHALL store `min`, `max`, `center`, and `size` in ship-local space, constructed from `min` and `max` via `BoundingBox::from_min_max(min, max)`, and SHALL provide `BoundingBox::corners(&self) -> [Vec2; 4]` as the single source of truth for its 4 local corners.

#### Scenario: Bounds built from grid with non-zero origin
- **WHEN** `BlockRegistry::bounds_from_grid` computes bounds for a `BlockGrid` whose minimum occupied cell is non-zero
- **THEN** `BoundingBox` stores `min = min_cell * GRID_SIZE`, `max = max_cell * GRID_SIZE`, `center = (min + max) * 0.5`, and `size = max - min`

#### Scenario: Local corners match min and max
- **WHEN** `corners()` is called on a `BoundingBox`
- **THEN** the returned 4 points equal `[(min.x, min.y), (max.x, min.y), (max.x, max.y), (min.x, max.y)]`

### Requirement: Ship center uses BoundingBox center
`ship_center(transform, bounds)` SHALL transform `bounds.center` into world space rather than `bounds.size * 0.5`.

#### Scenario: Rotated ship with offset bounds
- **WHEN** `ship_center` is called on a ship with a non-zero `bounds.min` and a rotated `Transform`
- **THEN** the returned world position equals `transform.transform_point(bounds.center.extend(0.0)).truncate()`

### Requirement: Marquee selection uses rotated world-space bounds
`update_selectable` SHALL transform `bounding_box.corners()` by the entity's `GlobalTransform` and test marquee rectangle overlap against the resulting world-space AABB.

#### Scenario: Marquee box overlaps rotated or offset ship
- **WHEN** a player drags a selection marquee over a ship whose `BoundingBox` has a non-zero `min` or non-zero rotation
- **THEN** the ship is selected if and only if the marquee rectangle overlaps the world-space AABB of the transformed `BoundingBox` corners
