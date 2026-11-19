## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Any ship can be selected by clicking
**Reason**: Replaced by the production `ship-inspector` capability (`InspectedShip`), which handles click selection in all builds and gates enemy ship selection to `dev`.
**Migration**: Use `InspectedShip` and `on_ship_click` from the `ship-inspector` capability.

### Requirement: Selection is a persistent multi-selection
**Reason**: Ship inspection now uses single-ship `InspectedShip(Option<Entity>)` rather than `ShipDebugSelection` (`EntityHashSet`).
**Migration**: Use `InspectedShip` for single-ship inspection; use `Ctrl+drag` (`SelectablePlugin`) for multi-ship fleet selection.

### Requirement: Debug text is shown only for selected ships
**Reason**: World-space `ShipUIRoot` `Text2d` is replaced by the screen-space Ship Inspector HUD panel, which renders dev-only fields (`Skill`, `Steering`, `AmmoState`) when `dev` is enabled.
**Migration**: Inspect ships via the bottom-right Ship Inspector HUD panel.

### Requirement: Debug selection is independent of gameplay selection
**Reason**: Replaced by `ship-inspector` selection requirements.
**Migration**: See `specs/ship-inspector/spec.md`.

### Requirement: Despawned ships leave the selection
**Reason**: Replaced by `ship-inspector` despawn handling on `InspectedShip`.
**Migration**: See `specs/ship-inspector/spec.md`.
