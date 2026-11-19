## ADDED Requirements

### Requirement: Top-right north-up minimap HUD
The `Game` scene SHALL display an always-on minimap anchored in the top-right corner of the viewport (present in both debug and release builds) and despawned via `DespawnOnExit(SceneState::Game)`. The minimap SHALL maintain a fixed north-up orientation (no map rotation) and clip markers to its circular bounds.

#### Scenario: Minimap spawns on entering Game and despawns on exit
- **WHEN** `SceneState` enters `Game`
- **THEN** the top-right minimap root is spawned with `DespawnOnExit(SceneState::Game)` and is visible even when debug HUD elements are disabled

#### Scenario: North-up coordinate mapping
- **WHEN** an enemy is located directly north (`+Y`) of the anchor ship and the anchor ship rotates 90 degrees
- **THEN** the enemy dot remains directly above the center of the minimap while only the player center arrow rotates

### Requirement: Minimap anchor fallback chain and player heading arrow
The minimap SHALL resolve its center anchor using the fallback chain: `ControlledShip.controlling` → `CameraTarget.following` → first alive player-team ship (ordered by `Ship.id`) → active `Camera2d` world position. When anchored to a ship, the minimap SHALL render a centered arrow rotated to match that ship's world heading and a radar range circle matching that ship's `Capability.max_radar_range`. When anchored to the free camera (no alive player-team ships), the center arrow and radar ring SHALL be hidden.

#### Scenario: Piloting a ship centers on it with rotating arrow and radar ring
- **WHEN** `ControlledShip.controlling` is `Some(player_ship)`
- **THEN** the minimap centers on `player_ship`, rotates the center arrow to `player_ship`'s heading, and sizes the radar ring from `player_ship`'s `Capability.max_radar_range`

#### Scenario: FreeLook with alive player ship falls back to followed or first ally
- **WHEN** `ControlledShip.controlling` is `None` and at least one player-team ship is alive
- **THEN** the minimap anchors to `CameraTarget.following` if it is an alive player-team ship, otherwise to the first alive player-team ship

#### Scenario: No alive player ships falls back to camera position
- **WHEN** zero player-team ships exist
- **THEN** the minimap anchors to the `Camera2d` position and hides the center arrow and radar ring

### Requirement: Minimap plots allies, live contacts, last-known ghosts, and alerts at fixed world scale
The minimap SHALL map world offsets using a fixed `world_radius` loaded from hot-reloadable config (`core/assets/config/base/minimap.ron`). It SHALL render:
- Allied player-team ships (other than the anchor ship) as allied-colored dots.
- Live detected enemies from player `TeamIntel.contacts` as enemy-colored dots.
- Decaying enemy positions from player `TeamIntel.last_known` as faded ghost dots whose opacity decreases with remaining TTL.
- Incoming-fire alerts from player `TeamIntel.alerts` as alert ping markers whose opacity decreases with remaining TTL.
Markers outside `world_radius` SHALL be hidden/clipped. Undetected enemies without a `last_known` entry SHALL NOT appear on the minimap.

#### Scenario: Radar ring scales with fixed world radius
- **WHEN** `world_radius` is `6000.0` and the anchor ship's `max_radar_range` is `2500.0`
- **THEN** the radar ring diameter is `(2500.0 / 6000.0)` of the minimap diameter

#### Scenario: Detected enemy, last-known ghost, and alert markers display properly
- **WHEN** player `TeamIntel` contains a live contact, a decaying `last_known` entry, and an active `TeamAlert` within `world_radius`
- **THEN** the minimap renders a solid dot for the live contact, a translucent ghost dot for the `last_known` entry, and a translucent ping for the alert

#### Scenario: Marker outside world radius is clipped
- **WHEN** a contact or ally is farther than `world_radius` from the minimap anchor
- **THEN** its marker is hidden and does not render outside the minimap bounds

### Requirement: Minimap configuration is data-driven and validated
`MinimapConfig` SHALL be loaded from `core/assets/config/base/minimap.ron` with hot-reload support and validated so that `size_px`, `world_radius`, `dot_size_px`, and `arrow_size_px` are strictly positive, `margin_px` is non-negative, and `max_markers >= 1`.

#### Scenario: Invalid minimap config is rejected
- **WHEN** `MinimapConfig` has `world_radius <= 0.0` or `max_markers == 0`
- **THEN** validation returns an explicit error
