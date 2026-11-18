# ship-debug Specification

## Purpose
TBD - created by archiving change ship-debug-gizmos. Update Purpose after archive.
## Requirements
### Requirement: Ship debug is compiled only with the dev feature

The ship debug selection, text, and gizmo systems SHALL exist only when the `dev` feature is enabled and SHALL run only while `SceneState::Game` is active.

#### Scenario: Production build
- **WHEN** the crate is built without the `dev` feature
- **THEN** no ship debug selection state, debug text, or debug gizmo system is present in the binary

#### Scenario: Outside the game scene
- **WHEN** the `dev` feature is enabled and the active scene is not `Game`
- **THEN** no ship click selection is applied and no ship debug text or gizmo is rendered

### Requirement: Any ship can be selected by clicking

In dev mode, a left click on a ship SHALL toggle that ship in the debug selection regardless of its team.

#### Scenario: Select an enemy ship
- **WHEN** the developer left-clicks an enemy ship
- **THEN** that ship is added to `ShipDebugSelection`

#### Scenario: Deselect a ship
- **WHEN** the developer left-clicks a ship that is already selected
- **THEN** that ship is removed from `ShipDebugSelection`

#### Scenario: Click on UI
- **WHEN** the developer clicks a HUD element above a ship
- **THEN** the ship under the HUD element is not selected

#### Scenario: Ctrl drag
- **WHEN** the developer holds `ControlLeft` while clicking a ship
- **THEN** the debug selection is not changed

#### Scenario: Clear selection
- **WHEN** the developer presses Escape
- **THEN** `ShipDebugSelection` becomes empty

### Requirement: Selection is a persistent multi-selection

`ShipDebugSelection` SHALL hold the set of all currently selected ships until a click, Escape, or despawn changes it.

#### Scenario: Several ships inspected at once
- **WHEN** the developer clicks three different ships
- **THEN** all three remain in `ShipDebugSelection` simultaneously

#### Scenario: Selection survives frames
- **WHEN** frames advance without a selection event
- **THEN** the selected set stays unchanged

### Requirement: Debug text is shown only for selected ships

Per-ship debug text SHALL be visible only for ships in `ShipDebugSelection`, and SHALL NOT be rebuilt every tick for unselected ships.

#### Scenario: Selected ship text
- **WHEN** a ship is selected
- **THEN** its `ShipUIRoot` shows `AiRuntime.current`, behavior, stats, cargo, and block ammo dumps

#### Scenario: Unselected ship text
- **WHEN** a ship is not selected
- **THEN** its `ShipUIRoot` text is hidden and its content is not recomputed each tick

#### Scenario: Selection change updates visibility
- **WHEN** a ship is selected or deselected
- **THEN** its text visibility reflects the new selection without reinserting text components

### Requirement: Debug gizmos are drawn only for selected ships

The debug gizmo renderer SHALL draw AI behavior geometry, hull/bounds with a core marker, heading/velocity/force, and turret aim rays only for ships in `ShipDebugSelection`.

#### Scenario: No selection
- **WHEN** `ShipDebugSelection` is empty
- **THEN** no ship debug gizmo is drawn

#### Scenario: Selected ship gizmos
- **WHEN** a ship is selected
- **THEN** gizmos are drawn from its `ShipBehavior`, `BoundingBox`, `Transform`, `Velocity`, `ExternalForce`, and the `TurretState` of blocks owned by that ship

#### Scenario: Deselected ship
- **WHEN** a ship is removed from the selection
- **THEN** its debug gizmos stop being drawn on the next render

### Requirement: Debug selection is independent of gameplay selection

Dev click selection SHALL NOT emit `EntitySelectEvent` or modify `ControlledShip` or `CameraTarget`.

#### Scenario: Inspect an enemy
- **WHEN** the developer selects an enemy ship by clicking
- **THEN** no `EntitySelectEvent` is emitted and the piloted ship and camera target are unchanged

#### Scenario: Gameplay drag select
- **WHEN** the developer box-selects player ships with Ctrl+drag
- **THEN** `ShipDebugSelection` stays unchanged

### Requirement: Despawned ships leave the selection

When a selected ship despawns, its entity SHALL be removed from `ShipDebugSelection`.

#### Scenario: Selected ship destroyed
- **WHEN** a selected ship is despawned
- **THEN** the entity is no longer in `ShipDebugSelection` and no gizmo or text is produced for it

### Requirement: AI debug geometry has a single source of truth

The geometry used by the AI behavior debug gizmos SHALL be computed by shared pure helpers used by both `update_ship_behavior` and the debug gizmo renderer.

#### Scenario: Steering and renderer agree
- **WHEN** a ship has an `Approach` or `GoAround` behavior
- **THEN** the renderer draws the same target and ring geometry that the steering system computes, without duplicating the math

