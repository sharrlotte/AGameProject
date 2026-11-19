# ship-inspector Specification

## Purpose
Production single-ship click selection (`InspectedShip`) and bottom-right Ship Inspector HUD with block sprite diagram and near-real-time stats, with enemy ship selection and debug fields gated to `dev`.

## Requirements

### Requirement: Single-ship inspection selection in Game scene
The system SHALL maintain an `InspectedShip(Option<Entity>)` resource in `SceneState::Game` across all builds, independent of `Ctrl+drag` (`EntitySelectEvent`), `CameraTarget`, and `ControlledShip` (except when triggered from `ShipList`). Selection queries in `apply_selection()` and `on_ship_click()` SHALL filter strictly with `With<Ship>`, and `apply_selection()` SHALL ignore any `ShipSelected` event whose target entity is not a `Ship`.

#### Scenario: Select a player ship in-world
- **WHEN** the user left-clicks a player ship (`team.is_player()`) without holding `ControlLeft` and not clicking a HUD element
- **THEN** `InspectedShip` is set to `Some(ship_entity)`

#### Scenario: Non-ship entity is ignored by apply_selection
- **WHEN** a `ShipSelected` event targets an entity without `Ship` (such as a block, projectile, or debris) even if it has `Team` and `Visibility`
- **THEN** `apply_selection()` skips the event and leaves `InspectedShip` unchanged

#### Scenario: Toggle off an already inspected ship
- **WHEN** the user left-clicks the ship that is currently in `InspectedShip`
- **THEN** `InspectedShip` becomes `None`

#### Scenario: Clear inspection on Escape
- **WHEN** the user presses `Escape` while `InspectedShip` is `Some`
- **THEN** `InspectedShip` becomes `None` and `Escape` is consumed before pausing

#### Scenario: Despawned ship clears inspection
- **WHEN** the entity currently stored in `InspectedShip` despawns
- **THEN** `InspectedShip` becomes `None`

#### Scenario: Select from ShipList
- **WHEN** the user clicks a ship button in `ShipList`
- **THEN** `InspectedShip` is set to `Some(ship_entity)` in addition to updating `CameraTarget` / `ControlledShip`

### Requirement: Enemy ship inspection is gated to dev builds
The system SHALL prevent selecting non-player ships (`!team.is_player()`) into `InspectedShip` in non-dev builds, while allowing any ship to be inspected when the `dev` feature is enabled.

#### Scenario: Production build ignores enemy click
- **WHEN** the crate is built without `dev` and the user left-clicks an enemy ship
- **THEN** `InspectedShip` remains unchanged

#### Scenario: Dev build inspects enemy ship
- **WHEN** the crate is built with `dev` and the user left-clicks an enemy ship
- **THEN** `InspectedShip` is set to `Some(enemy_entity)`

### Requirement: Ship Inspector HUD panel displays block images and live stats
When `InspectedShip` is `Some(entity)`, the system SHALL display a bottom-right HUD panel in `SceneState::Game` showing the ship's block sprite diagram and near-real-time stats; when `InspectedShip` is `None`, the panel SHALL be hidden.

#### Scenario: Block image diagram renders ship blocks
- **WHEN** a ship is selected into `InspectedShip`
- **THEN** the inspector panel renders each block in `BlockGrid` using its `BlockRegistry` sprite `ImageNode`s (`Feature::Texture` and `Feature::Turret`) positioned and oriented according to its placement

#### Scenario: Block diagram updates on structural change without per-tick rebuilds
- **WHEN** frames advance without `InspectedShip` changing or `ShipChanged` firing for the inspected ship
- **THEN** the block diagram UI nodes are NOT despawned and respawned, while block `ImageNode` colors update in-place to reflect current `Health`

#### Scenario: Structural change rebuilds block diagram
- **WHEN** `ShipChanged` fires for the currently inspected ship (e.g., a block is destroyed)
- **THEN** the block diagram is rebuilt to match the updated `BlockGrid`

#### Scenario: Near-real-time ship stats
- **WHEN** a ship is inspected
- **THEN** the HUD panel updates in-place on a short interval (~50–100ms) with Hull HP (`health`/`max_health`), Power (`power_production`, `power_demand`, `power_stored`/`power_capacity`, `power_supply`), Cargo (`cargo_used`/`cargo_capacity` and `cargo_items`), Mobility (`mass`, `thrust_power`, `rotate_speed`), Combat (`max_range`, `min_range`, `ammo_ratio`, `max_radar_range`), and AI (`Capability.role`, `Doctrine`, `Brain.current`)

#### Scenario: Dev-only debug stats in HUD
- **WHEN** the crate is built with `dev` and a ship is inspected
- **THEN** the HUD panel additionally displays `Skill`, `Steering` state, and block `Ammo`/`AmmoState` summaries
