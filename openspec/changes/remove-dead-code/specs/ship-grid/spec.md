## REMOVED Requirements

### Requirement: All placement changes go through ShipMutation
**Reason**: Runtime placement does not exist. The editor mutates its own design `ShipGrid` and validates through `check_placement_validity`; the only live-ship mutation is block destruction.
**Migration**: `ShipMutation` exposes `DestroyBlock` only. Overlap rejection remains specified by "Spawn and editor validate ship layouts".

## ADDED Requirements

### Requirement: Block destruction goes through ShipMutation
The system SHALL expose `ShipMutation::DestroyBlock` and apply mutations once per fixed tick through `apply_ship_mutations`; no system SHALL mutate a live ship's `ShipGrid` directly.

#### Scenario: Destruction pipeline
- **WHEN** a block is destroyed
- **THEN** the mutation removes every occupied cell, recomputes connected components, removes core-less fragments, updates stats and bounds, rebuilds the collider, and emits `ShipChanged`
