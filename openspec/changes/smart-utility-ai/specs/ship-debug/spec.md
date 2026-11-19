## MODIFIED Requirements

### Requirement: Debug text is shown only for selected ships

Per-ship debug text SHALL be visible only for ships in `ShipDebugSelection`, and SHALL NOT be rebuilt every tick for unselected ships.

#### Scenario: Selected ship text
- **WHEN** a ship is selected
- **THEN** its `ShipUIRoot` shows `Brain.current`, `Skill`, `Doctrine`, inferred `Role`, `Steering`, stats, cargo, and block ammo dumps

#### Scenario: Unselected ship text
- **WHEN** a ship is not selected
- **THEN** its `ShipUIRoot` text is hidden and its content is not recomputed each tick

#### Scenario: Selection change updates visibility
- **WHEN** a ship is selected or deselected
- **THEN** its text visibility reflects the new selection without reinserting text components

### Requirement: Debug gizmos are drawn only for selected ships

The debug gizmo renderer SHALL draw AI steering/radar geometry, hull/bounds with a core marker, heading/velocity/force, and turret aim rays only for ships in `ShipDebugSelection`.

#### Scenario: No selection
- **WHEN** `ShipDebugSelection` is empty
- **THEN** no ship debug gizmo is drawn

#### Scenario: Selected ship gizmos
- **WHEN** a ship is selected
- **THEN** gizmos are drawn from its `Steering`, `Capability` (`max_radar_range`), `BoundingBox`, `Transform`, `Velocity`, `ExternalForce`, and the `TurretState` of blocks owned by that ship

#### Scenario: Deselected ship
- **WHEN** a ship is removed from the selection
- **THEN** its debug gizmos stop being drawn on the next render

### Requirement: AI debug geometry has a single source of truth

The geometry used by the AI steering debug gizmos SHALL be computed by shared pure helpers used by both `steer` / `update_ship_steering` and the debug gizmo renderer.

#### Scenario: Steering and renderer agree
- **WHEN** a ship has an active `Steering` intent
- **THEN** the renderer draws the same desired heading and thrust vector geometry that the steering system computes, without duplicating the math
