## MODIFIED Requirements

### Requirement: Overdrive field visual
Every `Booster` block SHALL have an associated visual overdrive field indicating its coverage radius, spawned synchronously via an `On<Add, Booster>` observer and linked via an `OverdriveField(Entity)` component on the booster block. The field SHALL render as an orange circle with a dashed perimeter ring and no fill, computing `ring_ratio` from the booster's actual `Booster.radius` in both battle and Ship Editor scenes. In battle scenes, the field SHALL be visible if and only if the booster is powered (`BoosterState.powered == true`), not `Disabled`, and its owning ship is actively inspected (`InspectedShip`), for both player and opponent teams. In the ship editor scene, the field SHALL render for placed booster blocks and on the block placement cursor preview when a booster block is selected. The visual field SHALL be positioned at `LAYER_OVERDRIVE_FIELD = 70.0` over ship hull blocks (`z = 10.0`) and turret barrels (`z = 50.0`), and below projectiles (`z = 100.0`) and shield envelopes (`z = 160.0`).

#### Scenario: Powered booster displays field in battle
- **WHEN** a booster block on an inspected ship is powered and not disabled during battle
- **THEN** its overdrive field visual is rendered at $Z = 70.0$ centered at the booster block with radius equal to `Booster.radius` and ring thickness scaled to `Booster.radius`

#### Scenario: Unpowered or disabled booster hides field
- **WHEN** a booster block lacks power or gains the `Disabled` component
- **THEN** its overdrive field visual becomes hidden

#### Scenario: Editor displays field for placed booster with accurate ring ratio
- **WHEN** a booster block with `radius = 1028.0` is placed on a ship in the ship editor
- **THEN** its overdrive field visual is spawned via the shared observer/material path using `ring_ratio` computed for `1028.0` rather than a hardcoded 256px material

#### Scenario: Editor cursor preview displays field
- **WHEN** the player selects a booster block in the editor palette and hovers over the grid
- **THEN** the placement cursor visual includes the overdrive field preview with `ring_ratio` computed from `Booster.radius`
