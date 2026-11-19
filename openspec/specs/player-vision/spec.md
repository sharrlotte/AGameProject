# Player Vision

## Purpose

TBD - created by syncing change vision-and-minimap. Update Purpose after archive.

## Requirements
### Requirement: Enemy visibility is driven by shared player TeamIntel

The vision system SHALL evaluate enemy ship visibility from `TacticalBoard.teams[DEFAULT_TEAM_ID].intel.contacts` while in `SceneState::Game`. Every non-player ship present in player `intel.contacts` SHALL have `Visibility::Visible`, and every non-player ship absent from player `intel.contacts` SHALL have `Visibility::Hidden`. Player-team ships SHALL always remain `Visibility::Visible`.

#### Scenario: Undetected enemy is hidden
- **WHEN** an enemy ship is outside the `max_radar_range` of all player-team ships
- **THEN** it is absent from player `TeamIntel.contacts` and its entity `Visibility` is set to `Visibility::Hidden`

#### Scenario: Enemy enters any allied ship radar range
- **WHEN** an enemy ship enters `max_radar_range` of any alive player-team ship
- **THEN** it is published into player `TeamIntel.contacts` and its entity `Visibility` is set to `Visibility::Visible`

#### Scenario: Enemy leaves radar range
- **WHEN** a previously detected enemy ship exits `max_radar_range` of all player-team ships (or the spotting ally is destroyed)
- **THEN** it moves out of player `TeamIntel.contacts` and its entity `Visibility` returns to `Visibility::Hidden`

### Requirement: Undetected enemies do not leak through indicators or selection
Offscreen ship indicators (`draw_indicators`), entity selection, and dev ship click inspection SHALL exclude non-player ships that are not in player `TeamIntel.contacts` (or have `Visibility::Hidden`).

#### Scenario: Offscreen undetected enemy has no edge indicator
- **WHEN** an undetected enemy ship is outside the camera viewport
- **THEN** `draw_indicators` SHALL NOT draw an offscreen indicator arrow for that enemy

#### Scenario: Offscreen detected enemy shows edge indicator
- **WHEN** a detected enemy ship is outside the camera viewport
- **THEN** `draw_indicators` SHALL draw an offscreen indicator arrow for that enemy

#### Scenario: Clicking an undetected enemy does not select or inspect it
- **WHEN** the player clicks at the world coordinates of a `Visibility::Hidden` enemy ship
- **THEN** selection and dev ship debug UI SHALL NOT select or open inspection for that enemy

### Requirement: Projectiles remain visible regardless of shooter detection
Bullets, missiles, and laser beams SHALL remain `Visibility::Visible` even when fired by an undetected enemy ship.

#### Scenario: Undetected enemy fires from outside radar range
- **WHEN** an undetected enemy ship fires a projectile toward a player ship
- **THEN** the enemy ship remains `Visibility::Hidden` while the projectile renders visibly in flight
