# ship-debug Specification

## Purpose
Dev-only ship debug gizmo rendering driven by `InspectedShip`.

## Requirements
### Requirement: Ship debug is compiled only with the dev feature

The ship debug gizmo system SHALL exist only when the `dev` feature is enabled and SHALL run only while `SceneState::Game` is active.

#### Scenario: Production build
- **WHEN** the crate is built without the `dev` feature
- **THEN** no ship debug gizmo system is present in the binary

#### Scenario: Outside the game scene
- **WHEN** the `dev` feature is enabled and the active scene is not `Game`
- **THEN** no ship debug gizmo is rendered

### Requirement: Debug gizmos are drawn only for selected ships

The debug gizmo renderer SHALL draw AI steering/radar geometry, hull/bounds with a core marker, heading/velocity/force, and turret aim rays only for the ship in `InspectedShip`.

#### Scenario: No selection
- **WHEN** `InspectedShip` is `None`
- **THEN** no ship debug gizmo is drawn

#### Scenario: Selected ship gizmos
- **WHEN** `InspectedShip` is `Some(entity)`
- **THEN** gizmos are drawn from its `Steering`, `Capability` (`max_radar_range`), `BoundingBox`, `Transform`, `Velocity`, `ExternalForce`, and the `TurretState` of blocks owned by that ship

#### Scenario: Deselected ship
- **WHEN** `InspectedShip` becomes `None` or switches to another ship
- **THEN** debug gizmos for the previous ship stop being drawn on the next render

### Requirement: AI debug geometry has a single source of truth

The geometry used by the AI steering debug gizmos SHALL be computed by shared pure helpers used by both `steer` / `update_ship_steering` and the debug gizmo renderer.

#### Scenario: Steering and renderer agree
- **WHEN** a ship has an active `Steering` intent
- **THEN** the renderer draws the same desired heading and thrust vector geometry that the steering system computes, without duplicating the math
