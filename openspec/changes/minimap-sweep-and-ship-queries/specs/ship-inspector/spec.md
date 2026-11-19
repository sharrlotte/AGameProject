## MODIFIED Requirements

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
